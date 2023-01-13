use std::str::FromStr;

use anyhow::{Context, Result};
use chrono::{DateTime, Local};
use google_calendar3::api::{Event, EventDateTime};
use graphql_client::{GraphQLQuery, Response};
use reqwest;

use class_week_query::{ResponseData, Variables};

use self::class_week_query::ClassWeekQueryClassWeek;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/class_week.graphql",
    response_derives = "Debug"
)]
pub struct ClassWeekQuery;

#[derive(Debug)]
pub struct Lesson {
    pub id: Option<String>,
    pub subject: String,
    pub classroom: String,
    pub teacher: String,
    pub start: DateTime<Local>,
    pub end: DateTime<Local>,
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

impl PartialEq for Lesson {
    fn eq(&self, other: &Self) -> bool {
        self.subject == other.subject && self.start == other.start && self.end == other.end
    }
}

pub async fn get_timetable(variables: Variables) -> Result<ClassWeekQueryClassWeek> {
    let request_body = ClassWeekQuery::build_query(variables);

    let client = reqwest::Client::new();

    let res = client
        .post("https://v3.vegova.sync.si/graphql")
        .json(&request_body)
        .send()
        .await?;

    let response_body: Response<ResponseData> = res.json().await?;

    Ok(response_body.data.context("No timetable data")?.class_week)
}
