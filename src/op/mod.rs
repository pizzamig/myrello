use serde::{Deserialize, Serialize};
use serde_json::Error as JsonError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: u32,
    pub descr: String,
    pub priority: String,
    pub status: String,
    pub storypoints: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShowOneTask {
    pub name: String,
    pub id: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShowOneTaskReply {
    pub result: Task,
}

pub trait JsonCommand
where
    Self: Serialize,
{
    fn as_bytes(&self) -> Result<Vec<u8>, JsonError> {
        serde_json::to_vec(self)
    }
}

impl JsonCommand for ShowOneTask {}

pub trait JsonReply<'a>
where
    Self: Deserialize<'a>,
{
    fn from_bytes(slice: &'a [u8]) -> Result<Self, JsonError> {
        serde_json::from_slice(slice)
    }
}

impl JsonReply<'_> for ShowOneTaskReply {}

#[cfg(test)]
mod op_test {
    use super::*;

    #[test]
    fn test_jsoncommand() {
        let uut = ShowOneTask {
            name: "ShowOneTask".to_string(),
            id: 10,
        };
        let output: ShowOneTask = serde_json::from_slice(&uut.as_bytes().unwrap()).unwrap();

        assert_eq!(output.name, uut.name);
        assert_eq!(output.id, uut.id);
    }
}
