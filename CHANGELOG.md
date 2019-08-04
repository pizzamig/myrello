# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Added
- step: new subcommand to manage steps for tasks

### Changed
- show short: show also blocked tasks

## [0.3.1] 2019-04-05
### Added
- show: add subcommand done, to get a list of done tasks
- show: -t option, to show only one task (with all information)
- show: add subcommand short, to see only high priority and in_progress tasks

### Changed
- show: summary information is showed at the end

### Fix
- fix build with the stable toolchain (1.29)

## [0.3.0] 2018-09-25
### Added
- add support to story points
- add support to task references, in task-new and task-edit
- show: add -r option to toggle references 
- task-edit: to change/edit every attribute of a task
- task-prio: to increment the priority of a task
- task-block: to mark a task as blocked (status: block)
- task-start: to start to work on a task (status: start)
- show-backlog: to show all "todo" tasks
- show-work: to show all "in_progress" tasks

### Changed
- show: labels are always showed; -l option is now used as a filter and accept an argument
- task-new: task-add is not called task-new
- reference support changed the database schema
- show: complete review of the show subcommand
- show: -l can be use multiple team for finer label selection
- database-init: implemented option --force

### Removed
- task-status: superseeded by task-start, task-block, task-done and task-edit
- task-priority: superseeded by task-prio and task-edit

### Fix
- task-edit: option check was wronlgy too strict

## [0.2.0] 2018-08-20
### Added
- task-add: option -l to attach labels to a task
- task-add-label: new command to add a label to an existing task
- show: implement -l option, to show labels as well
- task-delete: new command to delete a task
- task-done: new command to complete a task
- task-status: new command to change status
- completion: new command to generate Zsh autocompletion function

### Changed
- the db schema was extended. No migration instruction provided
  priority, story points and status field were added

## [0.1.0] 2018-08-18
### Added
- database-init: initialize the sqlite database
- task-add: add a task
- show: show tasks
