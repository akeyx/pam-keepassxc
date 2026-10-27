use std::fs;

use nix::unistd::User;

use crate::MODULE_NAME;

pub struct UserConfig {
	database_path: String,
	keyfile_path: Option<String>,
}

fn parse_config(contents: &str) -> Option<UserConfig> {
	let mut database_path = None;
	let mut keyfile_path = None;

	for raw_line in contents.lines() {
		let line = raw_line.split('#').next().unwrap_or_default().trim();
		if line.is_empty() {
			continue;
		}
		let Some((key, val)) = line.split_once('=') else {
			continue;
		};
		let key = key.trim();
		let val = val.trim().trim_matches('"').trim_matches('\'').to_string();
		match key {
			"database_path" if !val.is_empty() => database_path = Some(val),
			"keyfile_path" if !val.is_empty() => keyfile_path = Some(val),
			_ => {}
		}
	}

	Some(UserConfig {
		database_path: database_path?,
		keyfile_path,
	})
}

pub fn user_config(user: &User) -> Option<UserConfig> {
	let config_path = user
		.dir
		.join(".config")
		.join("security")
		.join(MODULE_NAME)
		.with_extension("toml");

	parse_config(&fs::read_to_string(config_path).ok()?)
}

fn expand_path(user: &User, path: &str) -> String {
	let home_dir = user.dir.to_str().unwrap_or_default();

	let mut result = path.replace("$HOME", home_dir);
	if result.starts_with('~') {
		result.replace_range(0..1, home_dir);
	}
	result = result.replace("//", "/");

	result
}

pub fn database_path(user: &User, config: &UserConfig) -> String {
	expand_path(user, &config.database_path)
}

pub fn keyfile_path(user: &User, config: &UserConfig) -> Option<String> {
	config
		.keyfile_path
		.as_ref()
		.map(|path| expand_path(user, path))
}
