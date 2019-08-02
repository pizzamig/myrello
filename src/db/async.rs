use futures::{Async, Future, Poll};
use rusqlite::{params, Connection, Error};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

pub struct GetDb {
    filename: PathBuf,
}

impl Future for GetDb {
    type Item = Arc<Mutex<Connection>>;
    type Error = Error;
    fn poll(&mut self) -> Result<Async<Self::Item>, Error> {
        match crate::db::get_db(&self.filename) {
            Ok(c) => Ok(Async::Ready(Arc::new(Mutex::new(c)))),
            Err(e) => Err(e),
        }
    }
}

pub fn get_db_async(filename: &Path) -> GetDb {
    GetDb {
        filename: filename.to_path_buf(),
    }
}

pub struct GetPriorityIDbyTask {
    pub connection: Arc<Mutex<Connection>>,
    pub todo_id: u32,
}

impl Future for GetPriorityIDbyTask {
    type Item = u32;
    type Error = Error;

    fn poll(&mut self) -> Result<Async<Self::Item>, Error> {
        let conn = self.connection.lock().unwrap();
        let priority_id: u32 = conn.query_row(
            "SELECT priority_id
            FROM todos
            WHERE id = ?1;",
            params![&self.todo_id],
            |row| Ok(row.get(0)?),
        )?;
        Ok(Async::Ready(priority_id))
    }
}

struct IncreasePriority {
    connection: Arc<Mutex<Connection>>,
    todo_id: u32,
    priority_id_future: Option<Box<GetPriorityIDbyTask>>,
}

impl IncreasePriority {
    fn new(connection: Arc<Mutex<Connection>>, todo_id: u32) -> Self {
        IncreasePriority {
            connection,
            todo_id,
            priority_id_future: None,
        }
    }
}

impl Future for IncreasePriority {
    type Item = ();
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Error> {
        if self.priority_id_future.is_none() {
            self.priority_id_future = Some(Box::new(GetPriorityIDbyTask {
                connection: self.connection.clone(),
                todo_id: self.todo_id,
            }));
        }
        let mut priority_id = match self.priority_id_future.as_mut().unwrap().poll() {
            Ok(Async::Ready(v)) => v,
            Ok(Async::NotReady) => return Ok(Async::NotReady),
            Err(e) => return Err(e),
        };

        if priority_id != 1 {
            priority_id -= 1;
            let conn = self.connection.lock().unwrap();
            let rc = conn.execute(
                "UPDATE todos
                SET priority_id = ?1
                WHERE id = ?2;",
                params![&priority_id, &self.todo_id],
            )?;
            if rc != 1 {
                Err(Error::QueryReturnedNoRows)
            } else {
                Ok(Async::Ready(()))
            }
        } else {
            Ok(Async::Ready(()))
        }
    }
}
