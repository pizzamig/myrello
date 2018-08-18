enum Status {
    Todo,
    InProgress,
    Done,
    Block,
}

pub struct Task {
    pub id: u32,
    pub descr: String,
}
