# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Added
- task-edit: to change/edit the description of a task

### Changed
- show: labels are always showed; -l option is now used as a filter and accept an argument

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
