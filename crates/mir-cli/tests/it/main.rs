//! End-to-end tests share one program so cargo runs them in parallel instead of one program at a time.

mod color_output;
mod config_outside_project_root;
mod file_extensions;
mod project_files_restrict_composer_discovery;
mod psalm_error_level;
mod relative_config_ignore_files;
mod require_once_outside_autoload;
mod stale_baseline;
