use super::task;
use chrono::prelude::*;
use rusqlite::{Connection, Error};
use std::path::Path;

pub fn init(filename: &Path) -> Result<(), Error> {
    let c = Connection::open(filename)?;
    c.execute(
        "CREATE TABLE todos ( 
        id INTEGER PRIMARY KEY ASC,
        creation_date datetime,
        descr varchar(1024),
        priority_id INTEGER,
        status_id INTEGER,
        story_points INTEGER,
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
        label varchar(32),
        PRIMARY KEY (todo_id,label) );",
        &[],
    )?;
    c.execute(
        "CREATE TABLE status (
        id INTEGER PRIMARY KEY ASC,
        descr varchar(32));",
        &[],
    )?;
    c.execute(
        "CREATE TABLE priority (
        id INTEGER PRIMARY KEY ASC,
        descr varchar(16));",
        &[],
    )?;
    let priority = vec!["urgent", "high", "normal", "low", "miserable"];
    for p in priority {
        match c.execute(
            "INSERT INTO priority (descr)
            VALUES (?1 );",
            &[&p],
        ) {
            Ok(_) => (),
            Err(e) => return Err(e),
        };
    }
    let status = vec!["todo", "in_progress", "done", "block"];
    for s in status {
        match c.execute(
            "INSERT INTO status (descr)
            VALUES (?1 );",
            &[&s],
        ) {
            Ok(_) => (),
            Err(e) => return Err(e),
        };
    }
    Ok(())
}

pub fn get_db(filename: &Path) -> Result<Connection, Error> {
    Connection::open(filename)
}

pub fn add_task(filename: &Path, descr: &str) -> Result<u32, Error> {
    let db = get_db(filename)?;
    let creation_date: DateTime<Utc> = Utc::now();
    let creation_date_str = creation_date.format("%Y-%m-%d %H:%M:%S").to_string();
    let mut newdescr = String::from(descr.trim_right());
    newdescr.truncate(1024);
    let status: u32 = db.query_row(
        "SELECT id
        FROM status
        WHERE descr = \"todo\";",
        &[],
        |row| row.get(0),
    )?;
    let priority: u32 = db.query_row(
        "SELECT id
        FROM priority
        WHERE descr = \"normal\";",
        &[],
        |row| row.get(0),
    )?;
    match db.execute(
        "INSERT INTO todos (creation_date, descr, priority_id, status_id, story_points)
        VALUES (?1, ?2, ?3, ?4, 0);",
        &[&creation_date_str, &newdescr, &priority, &status],
    ) {
        Ok(_) => (),
        Err(e) => return Err(e),
    };
    let new_id: u32 = db.query_row(
        "SELECT id
        FROM todos
        WHERE creation_date = ?1;",
        &[&creation_date_str],
        |row| row.get(0),
    )?;
    Ok(new_id)
}

pub fn add_labels(filename: &Path, todo_id: u32, labels: &[String]) -> Result<(), Error> {
    let db = get_db(filename)?;
    for l in labels {
        let mut ll = String::from(l.trim());
        ll.truncate(256);
        match db.execute(
            "INSERT INTO todo_label (todo_id, label)
            VALUES (?1, ?2);",
            &[&todo_id, &ll],
        ) {
            Ok(_) => (),
            Err(e) => return Err(e),
        };
    }
    Ok(())
}

pub fn dbget_open_tasks(db: &Connection) -> Result<Vec<task::Task>, Error> {
    let mut stmt = db.prepare(
        "SELECT t.id,t.descr,p.descr,s.descr
        FROM todos t
        LEFT JOIN priority p ON p.id = t.priority_id
        LEFT JOIN status s ON s.id = t.status_id
        WHERE completion_date IS NULL;",
    )?;
    let query_iter = stmt.query_map(&[], |row| task::Task {
        id: row.get(0),
        descr: row.get(1),
        priority: row.get(2),
        status: row.get(3),
    })?;
    let rc = query_iter.map(|x| x.unwrap()).collect();
    Ok(rc)
}

pub fn get_open_tasks(filename: &Path) -> Result<Vec<task::Task>, Error> {
    let db = get_db(filename)?;
    dbget_open_tasks(&db)
}

pub fn dbget_labels(db: &Connection, todo_id: u32) -> Result<Vec<String>, Error> {
    let mut stmt = db.prepare(
        "SELECT label
        FROM todo_label
        WHERE todo_id = ?1;",
    )?;
    let query_iter = stmt.query_map(&[&todo_id], |row| row.get(0))?;
    let labels = query_iter.map(|x| x.unwrap()).collect();
    Ok(labels)
}

pub fn get_labels(filename: &Path, todo_id: u32) -> Result<Vec<String>, Error> {
    let db = get_db(filename)?;
    dbget_labels(&db, todo_id)
}

pub fn complete_task(filename: &Path, todo_id: u32) -> Result<(), Error> {
    let db = get_db(filename)?;
    let completion_date: DateTime<Utc> = Utc::now();
    let completion_date_str = completion_date.format("%Y-%m-%d %H:%M:%S").to_string();
    let rc = db.execute(
        "UPDATE todos
        SET completion_date = ?1
        WHERE id = ?2;",
        &[&completion_date_str, &todo_id],
    )?;
    if rc != 1 {
        Err(Error::QueryReturnedNoRows)
    } else {
        Ok(())
    }
}

pub fn delete_task(filename: &Path, todo_id: u32) -> Result<(), Error> {
    let db = get_db(filename)?;
    let rc = db.execute(
        "DELETE FROM todos
        WHERE id = ?1;",
        &[&todo_id],
    )?;
    db.execute(
        "DELETE FROM todo_label
        WHERE todo_id = ?1;",
        &[&todo_id],
    )?;
    if rc != 1 {
        Err(Error::StatementChangedRows(rc))
    } else {
        Ok(())
    }
}
