use anyhow::{Context, Result};
use api::{class_week::get_lessons, current_week::get_current_week};
use calendar::Calendar;
use chrono::{DateTime, Local, Timelike};
use lesson::Lesson;

mod api;
pub mod calendar;
mod lesson;

pub struct EACal<T: Calendar> {
    pub calendar: T,
}

impl<T: Calendar> EACal<T> {
    pub fn new(calendar: T) -> Self {
        EACal { calendar }
    }

    pub async fn sync_current(self, class: &str) -> Result<()> {
        let current_week = get_current_week().await?;
        Ok(self.sync(class, current_week).await?)
    }

    pub async fn sync(self, class: &str, week: u8) -> Result<()> {
        let lessons = get_lessons(class, week)
            .await
            .context("Failed to get lessons")?;

        let from = get_from(&lessons).context("Failed to get first lesson")?;
        let to = get_to(&lessons).context("Failed to get last lesson")?;

        let existing_lessons: Vec<Lesson> = self
            .calendar
            .list_lessons(from, to)
            .await
            .context("Failed to get lessons")?;

        for ex_lesson in &existing_lessons {
            if lessons.iter().any(|lesson| ex_lesson == lesson) {
                continue;
            }

            self.calendar
                .delete_lesson(ex_lesson)
                .await
                .context("Failed to delete lesson")?;

            println!("Deleted {}", ex_lesson.subject)
        }

        for lesson in &lessons {
            if existing_lessons.iter().any(|ex_lesson| ex_lesson == lesson) {
                continue;
            }

            let subject = lesson.subject.clone();

            self.calendar
                .add_lesson(lesson)
                .await
                .context("Failed to add lesson")?;

            println!("Added {}", subject);
        }

        Ok(())
    }
}

fn get_from(lessons: &Vec<Lesson>) -> Option<DateTime<Local>> {
    lessons
        .first()?
        .start
        .with_hour(0)?
        .with_minute(0)?
        .with_second(0)
}

fn get_to(lessons: &Vec<Lesson>) -> Option<DateTime<Local>> {
    lessons
        .last()?
        .end
        .with_hour(23)?
        .with_minute(59)?
        .with_second(59)
}
