use super::task;
use chrono::prelude::*;
use log::trace;
#[cfg(test)]
use proptest::{proptest, proptest_helper};
use rusqlite::{Connection, Error};
use std::path::Path;

pub fn delete_tables(db: &Connection) -> Result<(), Error> {
    db.execute("DROP TABLE IF EXISTS todos;", &[])?;
    db.execute("DROP TABLE IF EXISTS checklist_template;", &[])?;
    db.execute("DROP TABLE IF EXISTS todo_checklist;", &[])?;
    db.execute("DROP TABLE IF EXISTS todo_label;", &[])?;
    db.execute("DROP TABLE IF EXISTS refs;", &[])?;
    db.execute("DROP TABLE IF EXISTS status;", &[])?;
    db.execute("DROP TABLE IF EXISTS priority;", &[])?;
    Ok(())
}

pub fn init(filename: &Path, delete: bool) -> Result<(), Error> {
    let c = Connection::open(filename)?;
    if delete {
        delete_tables(&c)?
    }
    c.execute(
        "CREATE TABLE todos ( 
        id INTEGER PRIMARY KEY ASC,
        creation_date datetime,
        descr varchar(128),
        priority_id INTEGER,
        status_id INTEGER,
        refs_id INTEGER,
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
        "CREATE TABLE refs (
        id INTEGER PRIMARY KEY ASC,
        descr varchar(1024));",
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
    let mut newdescr = String::from(descr.trim_end());
    newdescr.truncate(128);
    let status: u32 = db.query_row(
        "SELECT id
        FROM status
        WHERE descr = \"todo\";",
        &[],
        |row| row.get(0),
    )?;
    let priority = dbget_priority_id(&db, "normal")?;
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

pub fn dbget_done_tasks(db: &Connection) -> Result<Vec<task::TaskDone>, Error> {
    let mut stmt = db.prepare(
        "SELECT t.id,t.descr,t.completion_date, t.story_points
        FROM todos t
        LEFT JOIN status s ON s.id = t.status_id
        WHERE s.descr = \"done\"
        ORDER BY completion_date ASC;",
    )?;
    let query_iter = stmt.query_map(&[], |row| task::TaskDone {
        id: row.get(0),
        descr: row.get(1),
        completion_date: row.get(2),
        storypoints: row.get_checked(3).unwrap_or(0),
    })?;
    let rc = query_iter.map(|x| x.unwrap()).collect();
    Ok(rc)
}

pub fn get_done_tasks(filename: &Path) -> Result<Vec<task::TaskDone>, Error> {
    let db = get_db(filename)?;
    dbget_done_tasks(&db)
}

pub fn dbget_open_tasks(db: &Connection) -> Result<Vec<task::Task>, Error> {
    let mut stmt = db.prepare(
        "SELECT t.id,t.descr,p.descr,s.descr,t.story_points
        FROM todos t
        LEFT JOIN priority p ON p.id = t.priority_id
        LEFT JOIN status s ON s.id = t.status_id
        WHERE completion_date IS NULL
        ORDER BY t.priority_id ASC;",
    )?;
    let query_iter = stmt.query_map(&[], |row| task::Task {
        id: row.get(0),
        descr: row.get(1),
        priority: row.get(2),
        status: row.get(3),
        storypoints: row.get_checked(4).unwrap_or(0),
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

pub fn get_refs(filename: &Path, todo_id: u32) -> Result<String, Error> {
    let db = get_db(filename)?;
    trace!("Query the refs_id from todos");
    let refs_id: u32 = db.query_row(
        "SELECT refs_id
        FROM todos
        WHERE id = ?1;",
        &[&todo_id],
        |row| row.get_checked(0).unwrap_or(0),
    )?;
    if refs_id == 0 {
        return Ok(String::new());
    }
    trace!("Query the descr from refs");
    let refs: String = db.query_row(
        "SELECT descr
        FROM refs
        WHERE id = ?1;",
        &[&refs_id],
        |row| row.get(0),
    )?;
    Ok(refs)
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

pub fn dbget_priority_id(db: &Connection, priority: &str) -> Result<u32, Error> {
    trace!("get priority id ({})", priority);
    let priority_id: u32 = db.query_row(
        "SELECT id
        FROM priority
        WHERE descr = ?1;",
        &[&priority],
        |row| row.get(0),
    )?;
    Ok(priority_id)
}

pub fn dbget_status_id(db: &Connection, status: &str) -> Result<u32, Error> {
    trace!("get status id ({})", status);
    let status_id: u32 = db.query_row(
        "SELECT id
        FROM status
        WHERE descr = ?1;",
        &[&status],
        |row| row.get(0),
    )?;
    Ok(status_id)
}

#[allow(dead_code)]
pub fn get_priority_id(filename: &Path, priority: &str) -> Result<u32, Error> {
    let db = get_db(filename)?;
    dbget_priority_id(&db, priority)
}

pub fn dbset_priority(db: &Connection, todo_id: u32, priority: &str) -> Result<(), Error> {
    let priority_id = dbget_priority_id(&db, priority)?;
    let rc = db.execute(
        "UPDATE todos
        SET priority_id = ?1
        WHERE id = ?2;",
        &[&priority_id, &todo_id],
    )?;
    if rc != 1 {
        Err(Error::QueryReturnedNoRows)
    } else {
        Ok(())
    }
}

pub fn set_priority(filename: &Path, todo_id: u32, priority: &str) -> Result<(), Error> {
    let db = get_db(filename)?;
    dbset_priority(&db, todo_id, priority)
}

pub fn dbset_status(db: &Connection, todo_id: u32, status: &str) -> Result<(), Error> {
    let status_id = dbget_status_id(&db, status)?;
    let rc = db.execute(
        "UPDATE todos
        SET status_id = ?1
        WHERE id = ?2;",
        &[&status_id, &todo_id],
    )?;
    if rc != 1 {
        Err(Error::QueryReturnedNoRows)
    } else {
        Ok(())
    }
}

pub fn set_status(filename: &Path, todo_id: u32, status: &str) -> Result<(), Error> {
    let db = get_db(filename)?;
    dbset_status(&db, todo_id, status)
}

pub fn dbset_descr(db: &Connection, todo_id: u32, descr: &str) -> Result<(), Error> {
    let mut newdescr = String::from(descr.trim_end());
    newdescr.truncate(128);
    let rc = db.execute(
        "UPDATE todos
        SET descr = ?1
        WHERE id = ?2;",
        &[&newdescr, &todo_id],
    )?;
    if rc != 1 {
        Err(Error::QueryReturnedNoRows)
    } else {
        Ok(())
    }
}

pub fn set_descr(filename: &Path, todo_id: u32, descr: &str) -> Result<(), Error> {
    let db = get_db(filename)?;
    dbset_descr(&db, todo_id, descr)
}

pub fn dbset_storypoint(db: &Connection, todo_id: u32, storypoint: u32) -> Result<(), Error> {
    let rc = db.execute(
        "UPDATE todos
        SET story_points = ?1
        WHERE id = ?2;",
        &[&storypoint, &todo_id],
    )?;
    if rc != 1 {
        Err(Error::QueryReturnedNoRows)
    } else {
        Ok(())
    }
}

pub fn set_storypoint(filename: &Path, todo_id: u32, storypoint: u32) -> Result<(), Error> {
    let db = get_db(filename)?;
    dbset_storypoint(&db, todo_id, storypoint)
}

pub fn dbincrease_priority(db: &Connection, todo_id: u32) -> Result<(), Error> {
    let priority_id: u32 = db.query_row(
        "SELECT priority_id
        FROM todos
        WHERE id = ?1;",
        &[&todo_id],
        |row| row.get(0),
    )?;
    if priority_id != 1 {
        let priority_id = priority_id - 1;
        let rc = db.execute(
            "UPDATE todos
            SET priority_id = ?1
            WHERE id = ?2;",
            &[&priority_id, &todo_id],
        )?;
        if rc != 1 {
            Err(Error::QueryReturnedNoRows)
        } else {
            Ok(())
        }
    } else {
        Ok(())
    }
}

pub fn increase_priority(filename: &Path, todo_id: u32) -> Result<(), Error> {
    let db = get_db(filename)?;
    dbincrease_priority(&db, todo_id)
}

pub fn dbset_reference(db: &Connection, todo_id: u32, reference: &str) -> Result<(), Error> {
    let mut newref = String::from(reference.trim_end());
    newref.truncate(1024);
    let rc = db.execute(
        "INSERT INTO refs (descr)
        VALUES (?1);",
        &[&newref],
    )?;
    if rc != 1 {
        return Err(Error::QueryReturnedNoRows);
    }
    let ref_id: u32 = db.query_row(
        "SELECT id
        FROM refs
        WHERE descr = ?1;",
        &[&newref],
        |row| row.get(0),
    )?;
    let rc = db.execute(
        "UPDATE todos
        SET refs_id = ?1
        WHERE id = ?2;",
        &[&ref_id, &todo_id],
    )?;
    if rc != 1 {
        Err(Error::QueryReturnedNoRows)
    } else {
        Ok(())
    }
}

pub fn set_reference(filename: &Path, todo_id: u32, reference: &str) -> Result<(), Error> {
    let db = get_db(filename)?;
    dbset_reference(&db, todo_id, reference)
}

#[cfg(test)]
mod test {
    use super::*;
    use assert_fs::prelude::*;
    use assert_fs::TempDir;

    #[test]
    fn test_init_1() {
        let temp = TempDir::new().unwrap();
        let dbfile = temp.child("dbtest");
        init(dbfile.path(), false).unwrap();
    }

    #[test]
    fn test_init_1b() {
        let temp = TempDir::new().unwrap();
        let dbfile = temp.child("dbtest");
        init(dbfile.path(), true).unwrap();
    }

    #[test]
    fn test_init_2() {
        let temp = TempDir::new().unwrap();
        let dbfile = temp.child("dbtest");
        dbfile.touch().unwrap();
        init(dbfile.path(), false).unwrap();
    }

    #[test]
    fn test_init_2b() {
        let temp = TempDir::new().unwrap();
        let dbfile = temp.child("dbtest");
        dbfile.touch().unwrap();
        init(dbfile.path(), true).unwrap();
    }

    #[test]
    fn test_init_3() {
        let temp = TempDir::new().unwrap();
        let dbfile = temp.child("dbtest");
        dbfile.touch().unwrap();
        init(dbfile.path(), false).unwrap();
        // force re-initialization
        init(dbfile.path(), true).unwrap();
    }

    #[test]
    fn test_init_3b() {
        let temp = TempDir::new().unwrap();
        let dbfile = temp.child("dbtest");
        dbfile.touch().unwrap();
        init(dbfile.path(), true).unwrap();
        // force re-initialization
        init(dbfile.path(), true).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_init_3c() {
        let temp = TempDir::new().unwrap();
        let dbfile = temp.child("dbtest");
        dbfile.touch().unwrap();
        init(dbfile.path(), true).unwrap();
        // force re-initialization
        init(dbfile.path(), false).unwrap();
    }

    #[test]
    fn test_get_db_1() {
        let temp = TempDir::new().unwrap();
        let dbfile = temp.child("dbtest");
        dbfile.touch().unwrap();
        init(dbfile.path(), true).unwrap();
        get_db(dbfile.path()).unwrap();
    }

    #[test]
    fn test_get_db_2() {
        let temp = TempDir::new().unwrap();
        let dbfile = temp.child("dbtest");
        get_db(dbfile.path()).unwrap();
    }

    proptest! {
        #[test]
        fn test_dbget_priority_1(ref s in "urgent|high|normal|low|miserable") {
            let temp = TempDir::new().unwrap();
            let dbfile = temp.child("dbtest");
            init(dbfile.path(), true).unwrap();
            let db = get_db(dbfile.path()).unwrap();
            dbget_priority_id(&db, s).unwrap();
        }
    }
    proptest! {
        #[test]
        fn test_dbget_priority_2(ref s in "[^uhnlm].*") {
            let temp = TempDir::new().unwrap();
            let dbfile = temp.child("dbtest");
            init(dbfile.path(), true).unwrap();
            let db = get_db(dbfile.path()).unwrap();
            assert_eq!( dbget_priority_id(&db, s).is_err(), true);
        }
    }

    proptest! {
        #[test]
        fn test_dbget_status_1(ref s in "todo|in_progress|done|block") {
            let temp = TempDir::new().unwrap();
            let dbfile = temp.child("dbtest");
            init(dbfile.path(), true).unwrap();
            let db = get_db(dbfile.path()).unwrap();
            dbget_status_id(&db, s).unwrap();
        }
    }
    proptest! {
        #[test]
        fn test_dbget_status_2(ref s in "[^tidb].*") {
            let temp = TempDir::new().unwrap();
            let dbfile = temp.child("dbtest");
            init(dbfile.path(), true).unwrap();
            let db = get_db(dbfile.path()).unwrap();
            assert_eq!( dbget_status_id(&db, s).is_err(), true);
        }
    }
}
