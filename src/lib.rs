#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
use std::panic::{catch_unwind, AssertUnwindSafe};

use anyhow::{anyhow, Result};
use log::{error, info, warn};
use nix::unistd::{ForkResult, User};
use pamsm::{pam_module, Pam, PamData, PamError, PamFlags, PamLibExt, PamServiceModule};
use secrecy::SecretString;

mod config;
mod dbus;
mod keyring;

use config::{user_config, UserConfig};
use dbus::{try_unlock, user_session_bus_path};

const MODULE_NAME: &str = "pam_keepassxc";

struct PamKeePassXC;

/**
 * Perform a double fork to detach the process and avoid zombie processes.
**/
fn double_fork() -> Result<ForkResult> {
	use nix::{
		sys::wait::waitpid,
		unistd::{close, fork, setsid},
	};
	match unsafe { fork() } {
		Ok(ForkResult::Parent { child, .. }) => {
			let _ = waitpid(child, None); // Wait for the first child process to exit.

			Ok(ForkResult::Parent { child })
		}
		Ok(ForkResult::Child) => {
			let _ = setsid();

			let _ = close(0);
			let _ = close(1);
			let _ = close(2);
			unsafe {
				let _ = nix::libc::syscall(nix::libc::SYS_close_range, 3usize, usize::MAX, 0usize);
			}

			// Fork again to create the grandchild process.
			match unsafe { fork() } {
				Ok(ForkResult::Child) => Ok(ForkResult::Child),
				_ => unsafe { nix::libc::_exit(0) },
			}
		}
		Err(err) => Err(anyhow!("{err}")),
	}
}

fn grandchild(
	wait_for_session: bool,
	user: &User,
	user_config: &UserConfig,
	pass_opt: Option<&SecretString>,
) -> Result<()> {
	use nix::unistd::{getgid, getuid, setgid, setresgid, setresuid, setuid};

	let _ = init_syslog(); // Reinitialize syslog for new PID

	let _ = setuid(getuid());
	let _ = setgid(getgid());

	let _ = setresgid(user.gid, user.gid, user.gid);
	let _ = setresuid(user.uid, user.uid, user.uid);

	let pass = if let Some(p) = pass_opt {
		if let Err(err) = keyring::store_password(user, p) {
			warn!("Failed to store password in user keyring: {err}");
		}
		p.clone()
	} else {
		keyring::load_password()?
	};

	try_unlock(wait_for_session, user, user_config, &pass)?;
	info!("Successfully unlocked KeePassXC database.");
	Ok(())
}

fn spawn_unlock_worker(
	wait_for_session: bool,
	user: &User,
	user_config: &UserConfig,
	pass_opt: Option<&SecretString>,
) {
	match double_fork() {
		Ok(ForkResult::Parent { child, .. }) => {
			info!("Spawned unlock worker (intermediate PID: {child})");
		}
		Ok(ForkResult::Child) => {
			if let Err(err) = grandchild(wait_for_session, user, user_config, pass_opt) {
				error!("KeePassXC unlock worker failed: {err}");
			}
			unsafe { nix::libc::_exit(0) };
		}
		Err(err) => {
			error!("Failed to fork unlock worker: {err}");
		}
	}
}

#[derive(Clone)]
struct SessionData {
	pass: Option<SecretString>, // Secret wrapper ensures data is zeroed on drop.
	unlocked_in_setcred: bool,
}

impl PamData for SessionData {}

fn init_syslog() -> Result<()> {
	use log::LevelFilter;
	use syslog::{BasicLogger, Facility, Formatter3164};

	let formatter = Formatter3164 {
		facility: Facility::LOG_USER,
		hostname: None,
		process: MODULE_NAME.into(),
		pid: std::process::id(),
	};

	let logger = syslog::unix(formatter)?;
	let _ = log::set_boxed_logger(Box::new(BasicLogger::new(logger)))
		.map(|()| log::set_max_level(LevelFilter::Info));

	Ok(())
}

fn resolve_user(pamh: &Pam) -> Result<User, PamError> {
	match pamh.get_user(None) {
		Ok(Some(u)) => {
			let Ok(name) = u.to_str() else {
				return Err(PamError::USER_UNKNOWN);
			};
			match User::from_name(name) {
				Ok(Some(u)) => Ok(u),
				_ => Err(PamError::USER_UNKNOWN),
			}
		}
		Ok(None) => Err(PamError::USER_UNKNOWN),
		Err(e) => Err(e),
	}
}

impl PamServiceModule for PamKeePassXC {
	fn authenticate(pamh: Pam, _flags: PamFlags, _args: Vec<String>) -> PamError {
		catch_unwind(AssertUnwindSafe(|| {
			let _ = init_syslog();

			let pass = match pamh.get_cached_authtok() {
				Ok(Some(p)) => p.to_str().ok().map(SecretString::from),
				Ok(None) | Err(_) => None,
			};

			let _ = unsafe {
				pamh.send_data(
					MODULE_NAME,
					SessionData {
						pass,
						unlocked_in_setcred: false,
					},
				)
			};

			PamError::SUCCESS
		}))
		.unwrap_or(PamError::IGNORE)
	}

	fn setcred(pamh: Pam, _flags: PamFlags, _args: Vec<String>) -> PamError {
		catch_unwind(AssertUnwindSafe(|| {
			let _ = init_syslog();
			let user = match resolve_user(&pamh) {
				Ok(u) => u,
				Err(e) => return e,
			};

			let Some(user_config) = user_config(&user) else {
				return PamError::IGNORE;
			};

			let data: SessionData =
				unsafe { pamh.retrieve_data(MODULE_NAME) }.unwrap_or(SessionData {
					pass: None,
					unlocked_in_setcred: false,
				});

			if let Some(ref pass) = data.pass {
				if let Err(err) = keyring::store_password(&user, pass) {
					warn!("Unable to store password in user keyring during setcred: {err}");
				}
			}

			if user_session_bus_path(&user).exists() {
				let _ = unsafe {
					pamh.send_data(
						MODULE_NAME,
						SessionData {
							pass: None,
							unlocked_in_setcred: true,
						},
					)
				};
				spawn_unlock_worker(false, &user, &user_config, data.pass.as_ref());
			}

			PamError::SUCCESS
		}))
		.unwrap_or(PamError::IGNORE)
	}

	#[cfg(feature = "session")]
	fn open_session(pamh: Pam, _flags: PamFlags, _args: Vec<String>) -> PamError {
		catch_unwind(AssertUnwindSafe(|| {
			let _ = init_syslog();
			let data: SessionData =
				unsafe { pamh.retrieve_data(MODULE_NAME) }.unwrap_or(SessionData {
					pass: None,
					unlocked_in_setcred: false,
				});
			let _ = unsafe {
				pamh.send_data(
					MODULE_NAME,
					SessionData {
						pass: None,
						unlocked_in_setcred: true,
					},
				)
			};

			if data.unlocked_in_setcred {
				return PamError::SUCCESS;
			}

			let user = match resolve_user(&pamh) {
				Ok(u) => u,
				Err(e) => return e,
			};

			let Some(user_config) = user_config(&user) else {
				return PamError::IGNORE;
			};

			info!("Trying to unlock keepassxc on session open...");
			spawn_unlock_worker(true, &user, &user_config, data.pass.as_ref());

			PamError::SUCCESS
		}))
		.unwrap_or(PamError::IGNORE)
	}

	#[cfg(not(feature = "session"))]
	fn open_session(_pamh: Pam, _flags: PamFlags, _args: Vec<String>) -> PamError {
		PamError::IGNORE
	}

	fn close_session(_pamh: Pam, _flags: PamFlags, _args: Vec<String>) -> PamError {
		PamError::IGNORE
	}
}

pam_module!(PamKeePassXC);

#[cfg(test)]
mod tests {
	use nix::unistd::geteuid;
	use secrecy::ExposeSecret;

	use super::*;

	#[test]
	fn test_keyring_roundtrip() {
		let user = User::from_uid(geteuid()).unwrap().unwrap();
		let secret = SecretString::from("test_pam_keepassxc_secret".to_string());
		keyring::store_password(&user, &secret).expect("store_password should succeed");
		let loaded = keyring::load_password().expect("load_password should succeed");
		assert_eq!(loaded.expose_secret(), "test_pam_keepassxc_secret");
	}

	#[test]
	fn test_dbus_verify_live() {
		let user = User::from_uid(geteuid()).unwrap().unwrap();
		if !dbus::user_session_bus_path(&user).exists() {
			return;
		}
		let mut conn = rustbus::RpcConn::connect_to_path(
			dbus::user_session_bus(&user).unwrap(),
			rustbus::connection::Timeout::Duration(std::time::Duration::from_secs(5)),
		)
		.unwrap();
		if let Ok(pid) = dbus::get_pid(&mut conn) {
			dbus::verify(&mut conn, pid).expect("verify should succeed on running KeePassXC");
		}
	}
}
