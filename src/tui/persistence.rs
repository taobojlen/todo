use crate::todo::{models::TodoList, writer};
use anyhow::Result;

pub trait Persistence {
    fn save_to_file(&self) -> Result<()>;
}

impl Persistence for TodoList {
    fn save_to_file(&self) -> Result<()> {
        writer::write_todo_file(self)
    }
}