pub mod db;
pub mod task;

use rusqlite::Connection;
use std::sync::{Arc, Mutex};

pub type MutConn = Arc<Mutex<Connection>>;
