# myrello

A local cli trello board clone

```
CREATE TABLE todos ( id INTEGER PRIMARY KEY ASC, creation_date datetime, descr varchar(128), priority_id INTEGER, status_id INTEGER, refs_id INTEGER, story_points INTEGER, completion_date datetime );
CREATE TABLE checklist_template ( id INTEGER , step INTEGER , descr varchar(1024) , PRIMARY KEY (id,step));
CREATE TABLE todo_checklist ( todo_id INTEGER, checklist_id INTEGER, checklist_step INTEGER, completion_date datetime, PRIMARY KEY (todo_id,checklist_id,checklist_step) );
CREATE TABLE todo_label ( todo_id INTEGER, label varchar(32), PRIMARY KEY (todo_id,lable) );
CREATE TABLE refs ( id INTEGER PRIMARY KEY ASC, descr varchar(1024) )
CREATE TABLE status ( id INTEGER PRIMARY KEY ASC, descr varchar(32) )
CREATE TABLE priority ( id INTEGER PRIMARY KEY ASC, descr varchar(16) )
CREATE TABLE steps ( todo_id INTEGER, steps_num INTEGER, descr varchar(1024), completion_date datetime )
```

predefined priorities:
1: urgent
2: high
3: normal
4: low
5: miserable

predefined status:
todo
in progress
done
blocked
