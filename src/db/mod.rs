pub mod r#async;
use super::task;
use super::task::{Step, Task};
use chrono::prelude::*;
use failure::Fail;
use log::trace;
use rusqlite::{params, Connection, Error};
use std::path::{Path, PathBuf};

#[derive(Fail, Debug)]
pub enum DbError {
    #[fail(display = "The db file has no valid parent directory")]
    DbFileNoParentDir,
    #[fail(display = "Failed to create the directory for the db file")]
    DbFileCreateParentDir,
}

pub fn dbfile_default() -> PathBuf {
    let default_dir = dirs::data_dir().unwrap_or_else(|| PathBuf::from("./"));
    default_dir.join("myrello.db")
}

pub fn dbdir_create(dbfile: &Path) -> Result<(), DbError> {
    if let Some(dbdir) = dbfile.parent() {
        if !dbdir.exists() {
            trace!("creating not existing default directory {:?}", dbdir);
            mkdirp::mkdirp(&dbdir)
                .map(|_| ())
                .map_err(|_| DbError::DbFileCreateParentDir)
        } else {
            Ok(())
        }
    } else {
        Err(DbError::DbFileNoParentDir)
    }
}

pub fn delete_tables(db: &Connection) -> Result<(), Error> {
    db.execute("DROP TABLE IF EXISTS todos;", params![])?;
    db.execute("DROP TABLE IF EXISTS checklist_template;", params![])?;
    db.execute("DROP TABLE IF EXISTS todo_checklist;", params![])?;
    db.execute("DROP TABLE IF EXISTS todo_label;", params![])?;
    db.execute("DROP TABLE IF EXISTS refs;", params![])?;
    db.execute("DROP TABLE IF EXISTS status;", params![])?;
    db.execute("DROP TABLE IF EXISTS priority;", params![])?;
    db.execute("DROP TABLE IF EXISTS steps;", params![])?;
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
        params![],
    )?;
    c.execute(
        "CREATE TABLE checklist_template (
        id INTEGER,
        step INTEGER,
        descr varchar(1024),
        PRIMARY KEY (id,step));",
        params![],
    )?;
    c.execute(
        "CREATE TABLE todo_checklist (
        todo_id INTEGER,
        checklist_id INTEGER,
        checklist_step INTEGER,
        completion_date datetime,
        PRIMARY KEY (todo_id,checklist_id,checklist_step) );",
        params![],
    )?;
    c.execute(
        "CREATE TABLE todo_label (
        todo_id INTEGER,
        label varchar(32),
        PRIMARY KEY (todo_id,label) );",
        params![],
    )?;
    c.execute(
        "CREATE TABLE refs (
        id INTEGER PRIMARY KEY ASC,
        descr varchar(1024));",
        params![],
    )?;
    c.execute(
        "CREATE TABLE status (
        id INTEGER PRIMARY KEY ASC,
        descr varchar(32));",
        params![],
    )?;
    c.execute(
        "CREATE TABLE priority (
        id INTEGER PRIMARY KEY ASC,
        descr varchar(16));",
        params![],
    )?;
    c.execute(
        "CREATE TABLE steps (
        todo_id INTEGER, steps_num INTEGER, descr varchar(1024), completion_date datetime );",
        params![],
    )?;
    let priority = vec!["urgent", "high", "normal", "low", "miserable"];
    for p in priority {
        match c.execute(
            "INSERT INTO priority (descr)
            VALUES (?1 );",
            params![&p],
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
            params![&s],
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

pub fn add_task(db: &Connection, descr: &str) -> Result<u32, Error> {
    let creation_date: DateTime<Utc> = Utc::now();
    let creation_date_str = creation_date.format("%Y-%m-%d %H:%M:%S").to_string();
    let mut newdescr = String::from(descr.trim_end());
    newdescr.truncate(128);
    let status: u32 = db.query_row(
        "SELECT id
        FROM status
        WHERE descr = \"todo\";",
        params![],
        |row| row.get(0),
    )?;
    let priority = get_priority_id(&db, "normal")?;
    db.execute(
        "INSERT INTO todos (creation_date, descr, priority_id, status_id, story_points)
        VALUES (?1, ?2, ?3, ?4, 0);",
        params![&creation_date_str, &newdescr, &priority, &status],
    )?;
    let new_id: u32 = db.query_row(
        "SELECT id
        FROM todos
        WHERE creation_date = ?1;",
        params![&creation_date_str],
        |row| row.get(0),
    )?;
    Ok(new_id)
}

pub fn add_labels(db: &Connection, todo_id: u32, labels: &[String]) -> Result<(), Error> {
    for l in labels {
        let mut ll = String::from(l.trim());
        ll.truncate(256);
        match db.execute(
            "INSERT INTO todo_label (todo_id, label)
            VALUES (?1, ?2);",
            params![&todo_id, &ll],
        ) {
            Ok(_) => (),
            Err(e) => return Err(e),
        };
    }
    Ok(())
}

pub fn get_done_tasks(db: &Connection) -> Result<Vec<task::TaskDone>, Error> {
    let mut stmt = db.prepare(
        "SELECT t.id,t.descr,t.completion_date, t.story_points
        FROM todos t
        LEFT JOIN status s ON s.id = t.status_id
        WHERE s.descr = \"done\"
        ORDER BY completion_date ASC;",
    )?;
    let query_iter = stmt.query_map(params![], |row| {
        Ok(task::TaskDone {
            id: row.get(0)?,
            descr: row.get(1)?,
            completion_date: row.get(2)?,
            storypoints: row.get(3).unwrap_or(0),
        })
    })?;
    let rc = query_iter.map(std::result::Result::unwrap).collect();
    Ok(rc)
}

pub fn get_open_tasks(db: &Connection) -> Result<Vec<Task>, Error> {
    let mut stmt = db.prepare(
        "SELECT t.id,t.descr,p.descr,s.descr,t.story_points
        FROM todos t
        LEFT JOIN priority p ON p.id = t.priority_id
        LEFT JOIN status s ON s.id = t.status_id
        WHERE completion_date IS NULL
        ORDER BY t.priority_id ASC;",
    )?;
    let query_iter = stmt.query_map(params![], |row| {
        Ok(task::Task {
            id: row.get(0)?,
            descr: row.get(1)?,
            priority: row.get(2)?,
            status: row.get(3)?,
            storypoints: row.get(4).unwrap_or(0),
        })
    })?;
    let rc = query_iter.map(std::result::Result::unwrap).collect();
    Ok(rc)
}

pub fn get_labels(db: &Connection, todo_id: u32) -> Result<Vec<String>, Error> {
    let mut stmt = db.prepare(
        "SELECT label
        FROM todo_label
        WHERE todo_id = ?1;",
    )?;
    let query_iter = stmt.query_map(&[&todo_id], |row| row.get(0))?;
    let labels = query_iter.map(std::result::Result::unwrap).collect();
    Ok(labels)
}

pub fn get_refs(db: &Connection, todo_id: u32) -> Result<String, Error> {
    trace!("Query the refs_id from todos");
    let refs_id: u32 = db.query_row(
        "SELECT refs_id
        FROM todos
        WHERE id = ?1;",
        params![&todo_id],
        |row| Ok(row.get(0).unwrap_or(0)),
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

pub fn complete_task(db: &Connection, todo_id: u32) -> Result<(), Error> {
    let completion_date: DateTime<Utc> = Utc::now();
    let completion_date_str = completion_date.format("%Y-%m-%d %H:%M:%S").to_string();
    let rc = db.execute(
        "UPDATE todos
        SET completion_date = ?1
        WHERE id = ?2;",
        params![&completion_date_str, &todo_id],
    )?;
    if rc != 1 {
        Err(Error::QueryReturnedNoRows)
    } else {
        Ok(())
    }
}

pub fn delete_task(db: &Connection, todo_id: u32) -> Result<(), Error> {
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

pub fn get_priority_id(db: &Connection, priority: &str) -> Result<u32, Error> {
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

pub fn get_status_id(db: &Connection, status: &str) -> Result<u32, Error> {
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

pub fn set_priority(db: &Connection, todo_id: u32, priority: &str) -> Result<(), Error> {
    let priority_id = get_priority_id(&db, priority)?;
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

pub fn set_status(db: &Connection, todo_id: u32, status: &str) -> Result<(), Error> {
    let status_id = get_status_id(&db, status)?;
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

pub fn set_descr(db: &Connection, todo_id: u32, descr: &str) -> Result<(), Error> {
    let mut newdescr = String::from(descr.trim_end());
    newdescr.truncate(128);
    let rc = db.execute(
        "UPDATE todos
        SET descr = ?1
        WHERE id = ?2;",
        params![&newdescr, &todo_id],
    )?;
    if rc != 1 {
        Err(Error::QueryReturnedNoRows)
    } else {
        Ok(())
    }
}

pub fn set_storypoint(db: &Connection, todo_id: u32, storypoint: u32) -> Result<(), Error> {
    let rc = db.execute(
        "UPDATE todos
        SET story_points = ?1
        WHERE id = ?2;",
        params![&storypoint, &todo_id],
    )?;
    if rc != 1 {
        Err(Error::QueryReturnedNoRows)
    } else {
        Ok(())
    }
}

pub fn increase_priority(db: &Connection, todo_id: u32) -> Result<(), Error> {
    let priority_id: u32 = db.query_row(
        "SELECT priority_id
        FROM todos
        WHERE id = ?1;",
        params![&todo_id],
        |row| row.get(0),
    )?;
    if priority_id != 1 {
        let priority_id = priority_id - 1;
        let rc = db.execute(
            "UPDATE todos
            SET priority_id = ?1
            WHERE id = ?2;",
            params![&priority_id, &todo_id],
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

pub fn add_step(db: &Connection, todo_id: u32, step_description: &str) -> Result<u32, Error> {
    let new_step: u32 = match db.query_row(
        "SELECT MAX(steps_num)
        FROM steps
        WHERE todo_id = ?1;",
        params![&todo_id],
        |row| row.get(0) as Result<u32, _>,
    ) {
        Ok(max_step) => max_step + 1,
        Err(_) => 0,
    };
    db.execute(
        "INSERT INTO steps (todo_id, steps_num, descr)
        VALUES (?1, ?2, ?3);",
        params![&todo_id, &new_step, &step_description],
    )?;
    Ok(new_step)
}

pub fn get_step(db: &Connection, todo_id: u32, step_id: u32) -> Result<Step, Error> {
    let mut stmt = db.prepare(
        "SELECT todo_id,steps_num,descr
        FROM steps
        WHERE todo_id = ?1 AND steps_num = ?2 ;",
    )?;
    let query_iter = stmt.query_map(params![&todo_id, &step_id], |row| {
        Ok(Step {
            todo_id: row.get(0)?,
            step_id: row.get(1)?,
            descr: row.get(2)?,
            completion_date: "".to_string(),
        })
    })?;
    let mut query_vec: Vec<_> = query_iter.map(std::result::Result::unwrap).collect();
    if query_vec.is_empty() {
        Err(Error::QueryReturnedNoRows)
    } else {
        Ok(query_vec.pop().unwrap())
    }
}

pub fn get_steps(db: &Connection, todo_id: u32) -> Result<Vec<Step>, Error> {
    let mut stmt = db.prepare(
        "SELECT todo_id,steps_num,descr
        FROM steps
        WHERE completion_date IS NULL AND todo_id = ?1
        ORDER BY steps_num ASC;",
    )?;
    let query_iter = stmt.query_map(params![&todo_id], |row| {
        Ok(Step {
            todo_id: row.get(0)?,
            step_id: row.get(1)?,
            descr: row.get(2)?,
            completion_date: "".to_string(),
        })
    })?;
    let result = query_iter.map(std::result::Result::unwrap).collect();
    Ok(result)
}

pub fn complete_step(db: &Connection, todo_id: u32, step_id: u32) -> Result<(), Error> {
    let completion_date: DateTime<Utc> = Utc::now();
    let completion_date_str = completion_date.format("%Y-%m-%d %H:%M:%S").to_string();
    let rc = db.execute(
        "UPDATE steps
        SET completion_date = ?1
        WHERE todo_id = ?2 AND steps_num = ?3;",
        params![&completion_date_str, &todo_id, &step_id],
    )?;
    if rc != 1 {
        Err(Error::QueryReturnedNoRows)
    } else {
        Ok(())
    }
}

pub fn complete_steps(db: &Connection, todo_id: u32) -> Result<(), Error> {
    let completion_date: DateTime<Utc> = Utc::now();
    let completion_date_str = completion_date.format("%Y-%m-%d %H:%M:%S").to_string();
    db.execute(
        "UPDATE steps
        SET completion_date = ?1
        WHERE completion_date IS NULL AND todo_id = ?2 ;",
        params![&completion_date_str, &todo_id],
    )?;
    Ok(())
}

pub fn delete_step(db: &Connection, todo_id: u32, step_id: u32) -> Result<(), Error> {
    db.execute(
        "DELETE FROM steps
        WHERE todo_id = ?1 AND steps_num = ?2;",
        &[&todo_id, &step_id],
    )?;
    Ok(())
}

pub fn delete_steps(db: &Connection, todo_id: u32) -> Result<(), Error> {
    db.execute(
        "DELETE FROM steps
        WHERE todo_id = ?1;",
        &[&todo_id],
    )?;
    Ok(())
}

pub fn set_reference(db: &Connection, todo_id: u32, reference: &str) -> Result<(), Error> {
    let mut newref = String::from(reference.trim_end());
    newref.truncate(1024);
    let rc = db.execute(
        "INSERT INTO refs (descr)
        VALUES (?1);",
        params![&newref],
    )?;
    if rc != 1 {
        return Err(Error::QueryReturnedNoRows);
    }
    let ref_id: u32 = db.query_row(
        "SELECT id
        FROM refs
        WHERE descr = ?1;",
        params![&newref],
        |row| row.get(0),
    )?;
    let rc = db.execute(
        "UPDATE todos
        SET refs_id = ?1
        WHERE id = ?2;",
        params![&ref_id, &todo_id],
    )?;
    if rc != 1 {
        Err(Error::QueryReturnedNoRows)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use assert_fs::prelude::*;
    use assert_fs::TempDir;
    use proptest::prelude::*;
    use proptest::{proptest, proptest_helper};

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
        #![proptest_config(ProptestConfig::with_cases(5))]
        #[test]
        fn test_get_priority_1(ref s in "urgent|high|normal|low|miserable") {
            let temp = TempDir::new().unwrap();
            let dbfile = temp.child("dbtest");
            init(dbfile.path(), true).unwrap();
            let db = get_db(dbfile.path()).unwrap();
            get_priority_id(&db, s).unwrap();
        }
    }
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(30))]
        #[test]
        fn test_get_priority_2(ref s in "[^uhnlm].*") {
            let temp = TempDir::new().unwrap();
            let dbfile = temp.child("dbtest");
            init(dbfile.path(), true).unwrap();
            let db = get_db(dbfile.path()).unwrap();
            assert_eq!( get_priority_id(&db, s).is_err(), true);
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(4))]
        #[test]
        fn test_get_status_1(ref s in "todo|in_progress|done|block") {
            let temp = TempDir::new().unwrap();
            let dbfile = temp.child("dbtest");
            init(dbfile.path(), true).unwrap();
            let db = get_db(dbfile.path()).unwrap();
            get_status_id(&db, s).unwrap();
        }
    }
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(30))]
        #[test]
        fn test_get_status_2(ref s in "[^tidb].*") {
            let temp = TempDir::new().unwrap();
            let dbfile = temp.child("dbtest");
            init(dbfile.path(), true).unwrap();
            let db = get_db(dbfile.path()).unwrap();
            assert_eq!( get_status_id(&db, s).is_err(), true);
        }
    }
}
