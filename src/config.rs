use std::fs;

use nix::unistd::User;
use serde::Deserialize;

use crate::MODULE_NAME;

#[derive(Deserialize)]
pub struct UserConfig {
	database_path: String,
	keyfile_path: Option<String>,
}

pub fn user_config(user: &User) -> Option<UserConfig> {
	let config_path = user
		.dir
		.join(".config")
		.join("security")
		.join(MODULE_NAME)
		.with_extension("toml");

	basic_toml::from_str(&fs::read_to_string(config_path).ok()?).ok()
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
