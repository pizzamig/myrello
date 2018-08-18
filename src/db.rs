use super::task;
use chrono::prelude::*;
use rusqlite::{Connection, Error};
use std::path::Path;

pub fn init(filename: &Path) -> Result<Connection, Error> {
    let c = Connection::open(filename)?;
    c.execute(
        "CREATE TABLE todos ( 
        id INTEGER PRIMARY KEY ASC,
        creation_date datetime,
        descr varchar(1024),
        in_progress boolean,
        completion_date datetime );",
        &[],
    )?;
    c.execute(
        "CREATE TABLE checklist_template (
        id INTEGER,
        step INTEGER,
        descr varchar(1024),
        PRIMARY KEY (id,step));",
        &[],
    )?;
    c.execute(
        "CREATE TABLE todo_checklist (
        todo_id INTEGER,
        checklist_id INTEGER,
        checklist_step INTEGER,
        completion_date datetime,
        PRIMARY KEY (todo_id,checklist_id,checklist_step) );",
        &[],
    )?;
    c.execute(
        "CREATE TABLE todo_label (
        todo_id INTEGER,
        label varchar(256),
        PRIMARY KEY (todo_id,label) );",
        &[],
    )?;
    Ok(c)
}

fn get_db(filename: &Path) -> Result<Connection, Error> {
    Connection::open(filename)
}

pub fn add_task(filename: &Path, descr: &str) -> Result<(), Error> {
    let db = get_db(filename)?;
    let creation_date: DateTime<Utc> = Utc::now();
    let creation_date_str = creation_date.format("%Y-%m-%d %H:%M:%S").to_string();
    let mut newdescr = String::from(descr.trim_right());
    newdescr.truncate(1024);
    match db.execute(
        "INSERT INTO todos (creation_date, descr, in_progress)
        VALUES (?1, ?2, false);",
        &[&creation_date_str, &newdescr],
    ) {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

pub fn get_open_tasks(filename: &Path) -> Result<Vec<task::Task>, Error> {
    let db = get_db(filename)?;
    let mut stmt = db.prepare(
        "SELECT id,descr
        FROM todos
        WHERE completion_date IS NULL;",
    )?;
    let query_iter = stmt.query_map(&[], |row| task::Task {
        id: row.get(0),
        descr: row.get(1),
    })?;
    let rc = query_iter.map(|x| x.unwrap()).collect();
    Ok(rc)
}
