use crate::task::ShowParams;
use crate::task::TimeWindow;
use structopt::StructOpt;
use structopt_flags::ForceFlag;

#[derive(Debug, StructOpt)]
pub enum Cmd {
    /// Show all
    #[structopt(name = "show")]
    Show(ShowOpt),
    /// Work on the task database
    #[structopt(name = "database")]
    Db(DbOpt),
    /// Work on tasks/todos
    #[structopt(name = "task")]
    Task(TaskOpt),
    /// Work on tasks steps
    #[structopt(name = "step")]
    Step(StepOpt),
    /// Generate autocompletion for zsh
    #[structopt(name = "completion")]
    Completion,
}

#[derive(Debug, StructOpt)]
pub struct DbOpt {
    #[structopt(subcommand)]
    pub cmd: DbCmd,
}

#[derive(Debug, StructOpt)]
pub enum DbCmd {
    /// Database initialization
    #[structopt(name = "init")]
    Init {
        #[structopt(flatten)]
        force_flag: ForceFlag,
    },
}

#[derive(Debug, StructOpt)]
pub struct ShowOpt {
    #[structopt(flatten)]
    pub show_opts: ShowCommonOpt,
    #[structopt(subcommand)]
    pub cmd: Option<ShowCmd>,
    /// The task id
    #[structopt(short = "t", long = "task")]
    pub task: Option<u32>,
}

#[derive(Debug, StructOpt)]
pub enum ShowCmd {
    /// Show tasks, except the completed ones
    #[structopt(name = "all")]
    All {
        #[structopt(flatten)]
        show_opts: ShowCommonOpt,
    },
    /// Show few tasks: high priority and/or in progress
    #[structopt(name = "short")]
    Short {
        #[structopt(flatten)]
        show_opts: ShowCommonOpt,
    },
    /// Show the backlog, all the tasks in todo
    #[structopt(name = "backlog")]
    Backlog {
        #[structopt(flatten)]
        show_opts: ShowCommonOpt,
    },
    /// Show the tasks that are "in_progress"
    #[structopt(name = "work")]
    Work {
        #[structopt(flatten)]
        show_opts: ShowCommonOpt,
    },
    /// Show the tasks that are "in_progress"
    #[structopt(name = "done")]
    Done {
        #[structopt(flatten)]
        show_opts: ShowCommonOpt,
        /// The time window desired
        /// Possible values are: today, yesterday, week, month
        #[structopt(short = "T", long = "time")]
        time_window: Option<TimeWindow>,
    },
}

#[derive(Debug, StructOpt, Default)]
pub struct ShowCommonOpt {
    /// Show fields normally hidden, like story points
    #[structopt(short = "H", long = "hidden")]
    pub hidden: bool,
    /// Select one or more label as filter
    #[structopt(short = "l", long = "label", raw(number_of_values = "1"))]
    pub labels: Vec<String>,
    /// Show references as well
    #[structopt(short = "r", long = "reference")]
    pub reference: bool,
    /// Show steps as well
    #[structopt(short = "s", long = "steps")]
    pub steps: bool,
}

impl ShowCommonOpt {
    pub fn as_show_params<'a>(&'a self, status: &'a str) -> ShowParams<'a> {
        ShowParams {
            label: &self.labels,
            status,
            reference: self.reference,
            storypoints: self.hidden,
            steps: self.steps,
        }
    }
    pub fn merge(&mut self, to_merge: &ShowCommonOpt) {
        self.hidden |= to_merge.hidden;
        self.reference |= to_merge.reference;
        to_merge
            .labels
            .iter()
            .for_each(|x| self.labels.push(x.to_string()));
    }
}

#[derive(Debug, StructOpt)]
pub struct TaskOpt {
    #[structopt(subcommand)]
    pub cmd: TaskCmd,
}

#[derive(Debug, StructOpt)]
pub enum TaskCmd {
    /// Create a new task
    #[structopt(name = "new")]
    New {
        /// attach one or more label to the task
        #[structopt(short = "l", long = "label", raw(number_of_values = "1"))]
        labels: Vec<String>,
        /// set a reference to the task
        #[structopt(short = "r", long = "reference")]
        reference: Option<String>,
        /// set a priority
        #[structopt(short = "p", long = "priority")]
        priority: Option<String>,
        /// the story points
        #[structopt(short = "S", long = "story-points")]
        storypoint: Option<u32>,
        /// The task description
        #[structopt(raw(required = "true"))]
        descr: Vec<String>,
    },
    /// Add a label to an existing task
    #[structopt(name = "add-label")]
    AddLabel {
        /// attach one or more label to the task
        #[structopt(short = "l", long = "label", raw(number_of_values = "1"))]
        labels: Vec<String>,
        /// The task id
        #[structopt(short = "t", long = "task")]
        task: u32,
    },
    /// Set a new description for the task
    #[structopt(name = "edit")]
    Edit {
        /// The task id
        #[structopt(short = "t", long = "task")]
        task: u32,
        /// the priority level
        #[structopt(short = "p", long = "priority")]
        priority: Option<String>,
        /// the task' status
        #[structopt(short = "s", long = "status")]
        status: Option<String>,
        /// the story points
        #[structopt(short = "S", long = "story-points")]
        storypoint: Option<u32>,
        /// set a reference to the task
        #[structopt(short = "r", long = "reference")]
        reference: Option<String>,
        /// The task description
        #[structopt()]
        descr: Vec<String>,
    },
    /// Close a task
    #[structopt(name = "done")]
    Done(OptTaskOnly),
    /// Start to work on a task
    #[structopt(name = "start")]
    Start(OptTaskOnly),
    /// Mark the task as blocked
    #[structopt(name = "block")]
    Block(OptTaskOnly),
    /// Delete a task
    #[structopt(name = "delete")]
    Delete(OptTaskOnly),
    /// Increase the priority of a task
    #[structopt(name = "prio")]
    Prio(OptTaskOnly),
}

#[derive(Debug, StructOpt)]
pub struct OptTaskOnly {
    /// The task id
    #[structopt(short = "t", long = "task")]
    pub task_id: u32,
}

#[derive(Debug, StructOpt)]
pub struct StepOpt {
    #[structopt(subcommand)]
    pub cmd: StepCmd,
}

#[derive(Debug, StructOpt)]
pub enum StepCmd {
    /// Add a new step to a task
    #[structopt(name = "add")]
    Add {
        /// The working task
        #[structopt(short = "t", long = "task")]
        task_id: u32,
        /// The task description
        #[structopt(raw(required = "true"))]
        descr: Vec<String>,
    },
    /// Mark a step as completed
    #[structopt(name = "done")]
    Done {
        /// The working task
        #[structopt(short = "t", long = "task")]
        task_id: u32,
        /// The step id
        #[structopt(short = "s", long = "step")]
        step_id: u32,
    },
    /// Delete a step
    #[structopt(name = "delete")]
    Delete {
        /// The working task
        #[structopt(short = "t", long = "task")]
        task_id: u32,
        /// The step id
        #[structopt(short = "s", long = "step")]
        step_id: u32,
    },
}

#[cfg(test)]
mod cli_opt_tests {
    use super::*;

    #[test]
    fn test_showcommonopt_default() {
        let mut uut = ShowCommonOpt::default();
        uut.merge(&ShowCommonOpt::default());
        assert!(!uut.hidden);
        assert!(!uut.reference);
        assert!(uut.labels.is_empty());
    }
    #[test]
    fn test_showcommonopt_true_values() {
        let mut uut = ShowCommonOpt::default();
        uut.hidden = true;
        uut.merge(&ShowCommonOpt {
            reference: true,
            ..ShowCommonOpt::default()
        });
        assert!(uut.hidden);
        assert!(uut.reference);
        assert!(uut.labels.is_empty());
    }
    #[test]
    fn test_showcommonopt_labels() {
        let mut label_uut = Vec::new();
        label_uut.push("label1".to_string());
        let mut uut = ShowCommonOpt {
            labels: label_uut,
            ..ShowCommonOpt::default()
        };
        uut.merge(&ShowCommonOpt::default());
        assert!(!uut.hidden);
        assert!(!uut.reference);
        assert_eq!(uut.labels.len(), 1);
        assert_eq!(uut.labels.pop().unwrap(), "label1".to_string());
    }

    // currently a feature too hard to implement at this level
    //    #[test]
    //    fn test_showcommonopt_labels_repeated() {
    //        let mut label_uut = Vec::new();
    //        label_uut.push("label1".to_string());
    //        let mut uut = ShowCommonOpt {
    //            labels: label_uut,
    //            ..ShowCommonOpt::default()
    //        };
    //        let mut label_uut = Vec::new();
    //        label_uut.push("label1".to_string());
    //        uut.merge(ShowCommonOpt {
    //            labels: label_uut,
    //            ..ShowCommonOpt::default()
    //        });
    //        assert!(!uut.hidden);
    //        assert!(!uut.reference);
    //        assert_eq!(uut.labels.len(), 1);
    //        assert_eq!(uut.labels.pop().unwrap(), "label1".to_string());
    //    }
}
