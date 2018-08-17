enum Status {
    Todo,
    InProgress,
    Done,
    Block,
}

struct Task {
    title: String,
    status: Status,
}
