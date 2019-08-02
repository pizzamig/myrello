pub mod cli_opt;
pub mod db;
pub mod op;
pub mod task;

use rusqlite::Connection;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

pub type MutConn = Arc<Mutex<Connection>>;
pub type LabelList = HashSet<String>;

pub fn labellist_from_vec(labels: Vec<String>) -> LabelList {
    let mut rv = LabelList::new();
    for l in labels {
        if !rv.contains(&l) {
            rv.insert(l);
        }
    }
    rv
}

//use std::convert::Infallible;
//use std::str::FromStr;
//impl FromStr for LabelList {
//    type Err = Infallible;
//    fn from_str(s: &str) -> Result<Self, Self::Err> {
//        let result = HashSet::new();
//        result.insert(s.to_string());
//        Ok(result)
//    }
//}
