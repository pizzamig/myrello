//enum Status {
//    Todo,
//    InProgress,
//    Done,
//    Block,
//}

#[derive(Debug)]
pub struct Task {
    pub id: u32,
    pub descr: String,
}

#[derive(Debug)]
pub struct TaskLabel {
    pub id: u32,
    pub descr: String,
    pub label: Vec<String>,
}
