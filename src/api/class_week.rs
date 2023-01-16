use anyhow::{Context, Result};
use chrono::{Datelike, Local, TimeZone, Timelike};
use graphql_client::{GraphQLQuery, Response};
use reqwest;

use class_week_query::{ResponseData, Variables};

use crate::lesson::Lesson;

use self::class_week_query::{
    ClassWeekQueryClassWeek, ClassWeekQueryClassWeekDaysLessons,
    ClassWeekQueryClassWeekScheduleDefinitions,
};

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/class_week.graphql",
    response_derives = "Debug"
)]
pub struct ClassWeekQuery;

pub async fn get_lessons(name: &str, week: u8) -> Result<Vec<Lesson>> {
    let timetable = get_timetable(Variables {
        name: name.to_owned(),
        week: week.into(),
    })
    .await?;

    let definitions = timetable.schedule_definitions;

    let mut lessons = Vec::new();

    for day in timetable.days {
        for (lesson_index, lesson) in day.lessons.into_iter().enumerate() {
            for details in lesson {
                let lesson =
                    parse_details(details, day.date.to_owned(), &definitions[lesson_index])?;
                lessons.push(lesson);
            }
        }
    }

    lessons.sort_by(|a, b| a.start.cmp(&b.start));

    Ok(lessons)
}

async fn get_timetable(variables: Variables) -> Result<ClassWeekQueryClassWeek> {
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

fn parse_details(
    details: ClassWeekQueryClassWeekDaysLessons,
    date: String,
    def: &ClassWeekQueryClassWeekScheduleDefinitions,
) -> Result<Lesson> {
    let date_parts = date
        .split(".")
        .map(|p| p.trim())
        .filter(|p| !p.is_empty())
        .collect::<Vec<&str>>();

    let date = chrono::Local
        .with_ymd_and_hms(
            Local::now().year(),
            date_parts[1].parse::<u32>()?,
            date_parts[0].parse::<u32>()?,
            0,
            0,
            0,
        )
        .earliest()
        .context("Failed to parse date")?;

    let start_parts = def.from.split(":").collect::<Vec<&str>>();
    let start_date = date
        .clone()
        .with_hour(start_parts[0].parse::<u32>()?)
        .context("Failed to set hour")?
        .with_minute(start_parts[1].parse::<u32>()?)
        .context("Failed to set minute")?;

    let end_parts = def.to.split(":").collect::<Vec<&str>>();
    let end_date = date
        .with_hour(end_parts[0].parse::<u32>()?)
        .context("Failed to set hour")?
        .with_minute(end_parts[1].parse::<u32>()?)
        .context("Failed to set minute")?;

    let lesson = Lesson {
        id: None,
        subject: details.name,
        classroom: details.room,
        teacher: details.teacher,
        start: start_date,
        end: end_date,
    };

    Ok(lesson)
}
