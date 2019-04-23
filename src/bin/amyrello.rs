use exitfailure::ExitFailure;
use myrello::db;
use std::path::Path;
use tokio::prelude::*;

fn main() -> Result<(), ExitFailure> {
    let fut = crate::db::get_db_async(Path::new("~/.local/share/myrello.db"))
        .and_then(move |connection| db::GetPriorityIDbyTask {
            connection: &connection,
            task_id: 1,
        })
        .and_then(|priority| {
            println!("the task 1 has priority {}", priority);
            Ok(())
        })
        .map_err(|e| eprintln!("{:?}", e));

    tokio::run(fut);

    Ok(())
}
