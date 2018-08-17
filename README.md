CREATE TABLE todos ( id INTEGER PRIMARY KEY ASC, creation_date datetime, descr varchar(1024), in_progress boolean, completion_date datetime );
CREATE TABLE checklist_template ( id INTEGER , step INTEGER , descr varchar(1024) , PRIMARY KEY (id,step));
CREATE TABLE todo_checklist ( todo_id INTEGER, checklist_id INTEGER, checklist_step INTEGER, completion_date datetime, PRIMARY KEY (todo_id,checklist_id,checklist_step) );
CREATE TABLE todo_label ( todo_id INTEGER, label varchar(256), PRIMARY KEY (todo_id,lable) );
