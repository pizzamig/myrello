use exitfailure::ExitFailure;
use myrello::db;
use std::path::Path;
use tokio::prelude::*;

fn main() -> Result<(), ExitFailure> {
    let fut =
        db::r#async::get_db_async(Path::new("/usr/home/lpizzamiglio/.local/share/myrello.db"))
            .and_then(move |connection| db::r#async::GetPriorityIDbyTask {
                connection,
                todo_id: 229,
            })
            .and_then(|priority| {
                println!("the task 1 has priority {}", priority);
                Ok(())
            })
            .map_err(|e| eprintln!("{:?}", e));

    tokio::run(fut);

    Ok(())
}
