mod json_db;
mod types;
mod utils;

pub use json_db::JsonDB;
pub use types::{Date, Name, Status, ToDo, User};
pub use utils::{fake_it, get_key_chain_value, get_nested_value};
