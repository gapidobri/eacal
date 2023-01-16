use std::str::FromStr;

use anyhow::{Context, Result};
use async_trait::async_trait;
use chrono::{DateTime, Local};
use google_calendar3::{
    api::{Event, EventDateTime},
    CalendarHub,
};
use hyper::client::HttpConnector;
use hyper_rustls::HttpsConnector;

use crate::lesson::Lesson;

use super::Calendar;

pub struct GoogleCalendar {
    hub: CalendarHub<HttpsConnector<HttpConnector>>,
    calendar_id: String,
}

impl GoogleCalendar {
    pub fn new(hub: CalendarHub<HttpsConnector<HttpConnector>>, calendar_id: String) -> Self {
        GoogleCalendar { calendar_id, hub }
    }
}

#[async_trait]
impl Calendar for GoogleCalendar {
    async fn list_lessons(
        &self,
        from: DateTime<Local>,
        to: DateTime<Local>,
    ) -> Result<Vec<Lesson>> {
        let (_, events) = self
            .hub
            .events()
            .list(self.calendar_id.as_str())
            .time_min(from.to_rfc3339().as_str())
            .time_max(to.to_rfc3339().as_str())
            .doit()
            .await?;

        let events = events.items.context("No lessons found")?;

        let lessons = events
            .into_iter()
            .filter_map(|e| TryInto::<Lesson>::try_into(e).ok())
            .collect();

        Ok(lessons)
    }

    async fn add_lesson(&self, lesson: &Lesson) -> Result<()> {
        self.hub
            .events()
            .insert(lesson.clone().into(), self.calendar_id.as_str())
            .doit()
            .await?;

        Ok(())
    }

    async fn delete_lesson(&self, lesson: &Lesson) -> Result<()> {
        self.hub
            .events()
            .delete(
                self.calendar_id.as_str(),
                lesson.id.as_ref().context("Lesson has no id")?.as_str(),
            )
            .doit()
            .await?;

        Ok(())
    }
}

impl TryFrom<Event> for Lesson {
    type Error = anyhow::Error;

    fn try_from(e: Event) -> Result<Lesson> {
        Ok(Lesson {
            id: e.id,
            subject: e.summary.context("summary field is missing")?,
            classroom: e.location.context("location field is missing")?,
            teacher: String::new(),
            start: DateTime::<Local>::from_str(
                e.start
                    .context("start time missing")?
                    .date_time
                    .context("date_time field is missing")?
                    .as_str(),
            )?,
            end: DateTime::<Local>::from_str(
                e.end
                    .context("end time is missing")?
                    .date_time
                    .context("date_time field is missing")?
                    .as_str(),
            )?,
        })
    }
}

impl Into<Event> for Lesson {
    fn into(self) -> Event {
        Event {
            summary: Some(self.subject),
            location: Some(self.classroom),
            start: Some(EventDateTime {
                date_time: Some(self.start.to_rfc3339()),
                ..EventDateTime::default()
            }),
            end: Some(EventDateTime {
                date_time: Some(self.end.to_rfc3339()),
                ..EventDateTime::default()
            }),
            ..Event::default()
        }
    }
}
