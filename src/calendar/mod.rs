use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Local};

use crate::lesson::Lesson;

pub mod google_calendar;

#[async_trait]
pub trait Calendar {
    async fn list_lessons(&self, from: DateTime<Local>, to: DateTime<Local>)
        -> Result<Vec<Lesson>>;
    async fn add_lesson(&self, lesson: &Lesson) -> Result<()>;
    async fn delete_lesson(&self, lesson: &Lesson) -> Result<()>;
}
