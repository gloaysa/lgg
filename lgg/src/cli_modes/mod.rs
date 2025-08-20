mod cli_mode;
mod edit_mode;
mod read_mode;
mod use_color;
mod write_mode;

pub use cli_mode::CliModeResult;
pub use edit_mode::edit_mode;
pub use read_mode::read_mode;
pub use write_mode::write_mode;
