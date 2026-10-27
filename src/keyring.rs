use std::ffi::CStr;

use anyhow::{anyhow, Result};
use nix::{
	libc::{c_char, c_long, c_void, syscall, SYS_add_key, SYS_keyctl},
	sys::wait::waitpid,
	unistd::{fork, geteuid, setresgid, setresuid, ForkResult, User},
};
use secrecy::{ExposeSecret, SecretString};

const KEY_SPEC_USER_KEYRING: c_long = -4;
const KEYCTL_SEARCH: c_long = 10;
const KEYCTL_READ: c_long = 11;

const KEY_TYPE: &CStr = c"user";
const KEY_DESC: &CStr = c"pam_keepassxc";

fn add_key_current_user(secret: &str) -> Result<()> {
	let ret = unsafe {
		syscall(
			SYS_add_key,
			KEY_TYPE.as_ptr(),
			KEY_DESC.as_ptr(),
			secret.as_ptr().cast::<c_void>(),
			secret.len(),
			KEY_SPEC_USER_KEYRING,
		)
	};
	if ret < 0 {
		return Err(anyhow!(
			"add_key failed: {}",
			std::io::Error::last_os_error()
		));
	}
	Ok(())
}

pub fn store_password(user: &User, pass: &SecretString) -> Result<()> {
	if geteuid() == user.uid {
		return add_key_current_user(pass.expose_secret());
	}

	match unsafe { fork() } {
		Ok(ForkResult::Parent { child }) => {
			let _ = waitpid(child, None);
			Ok(())
		}
		Ok(ForkResult::Child) => {
			let _ = setresgid(user.gid, user.gid, user.gid);
			let _ = setresuid(user.uid, user.uid, user.uid);
			let code = i32::from(add_key_current_user(pass.expose_secret()).is_err());
			unsafe { nix::libc::_exit(code) };
		}
		Err(e) => Err(anyhow!("fork failed while storing key: {e}")),
	}
}

pub fn load_password() -> Result<SecretString> {
	let key_id = unsafe {
		syscall(
			SYS_keyctl,
			KEYCTL_SEARCH,
			KEY_SPEC_USER_KEYRING,
			KEY_TYPE.as_ptr(),
			KEY_DESC.as_ptr(),
			0 as c_long,
		)
	};
	if key_id < 0 {
		return Err(anyhow!(
			"keyctl search failed: {}",
			std::io::Error::last_os_error()
		));
	}

	let len = unsafe {
		syscall(
			SYS_keyctl,
			KEYCTL_READ,
			key_id,
			std::ptr::null_mut::<c_char>(),
			0 as c_long,
		)
	};
	if len <= 0 {
		return Err(anyhow!("key in user keyring is empty or unreadable"));
	}

	let usize_len = usize::try_from(len)?;
	let mut buf = vec![0u8; usize_len];
	let read_len = unsafe {
		syscall(
			SYS_keyctl,
			KEYCTL_READ,
			key_id,
			buf.as_mut_ptr().cast::<c_char>(),
			len,
		)
	};
	if read_len < 0 {
		buf.fill(0);
		return Err(anyhow!(
			"keyctl read failed: {}",
			std::io::Error::last_os_error()
		));
	}

	buf.truncate(usize::try_from(read_len)?);
	let s = String::from_utf8(buf.clone()).map_err(|_| {
		buf.fill(0);
		anyhow!("keyring password is not valid UTF-8")
	})?;
	buf.fill(0);
	Ok(SecretString::from(s))
}
