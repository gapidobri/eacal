use std::env;

use anyhow::{Context, Result};
use chrono::{Datelike, Local, TimeZone, Timelike};
use class_week::{
    class_week_query::{
        self, ClassWeekQueryClassWeekDaysLessons, ClassWeekQueryClassWeekScheduleDefinitions,
    },
    Lesson,
};
use dotenv::dotenv;
use google_calendar3::{oauth2, CalendarHub};

mod class_week;
mod current_week;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let lessons = match get_lessons(String::from("R4C"), 20).await {
        Ok(lessons) => lessons,
        Err(err) => {
            println!("{}", err);
            return;
        }
    };

    let service_account_key = match oauth2::read_service_account_key("credentials.json").await {
        Ok(key) => key,
        Err(err) => {
            println!("{}", err);
            return;
        }
    };

    let auth = oauth2::ServiceAccountAuthenticator::builder(service_account_key)
        .build()
        .await
        .expect("Failed to build service account authenticator");

    let hub = CalendarHub::new(
        hyper::Client::builder().build(
            hyper_rustls::HttpsConnectorBuilder::new()
                .with_native_roots()
                .https_or_http()
                .enable_http1()
                .enable_http2()
                .build(),
        ),
        auth,
    );

    let calendar_id = env::var("CALENDAR_ID").expect("CALENDAR_ID not set");

    let existing_events = match hub.events().list(calendar_id.as_str()).doit().await {
        Ok((_, events)) => events.items.unwrap_or(vec![]),
        Err(_) => {
            println!("Failed to list existing events");
            return;
        }
    };

    let week_start = lessons
        .first()
        .unwrap()
        .start
        .with_hour(0)
        .unwrap()
        .with_minute(0)
        .unwrap();
    let week_end = lessons
        .last()
        .unwrap()
        .end
        .with_hour(23)
        .unwrap()
        .with_minute(59)
        .unwrap();

    let existing_lessons = existing_events
        .into_iter()
        .filter_map(|e| TryInto::<Lesson>::try_into(e).ok())
        .filter(|l| l.start > week_start && l.end < week_end)
        .collect::<Vec<Lesson>>();

    for ex_lesson in &existing_lessons {
        if lessons.iter().any(|lesson| ex_lesson == lesson) {
            continue;
        }

        hub.events()
            .delete(calendar_id.as_str(), ex_lesson.id.clone().unwrap().as_str())
            .doit()
            .await
            .unwrap();

        println!("Deleted {}", ex_lesson.subject)
    }

    for lesson in lessons {
        if existing_lessons
            .iter()
            .any(|ex_lesson| ex_lesson == &lesson)
        {
            continue;
        }

        let subject = lesson.subject.clone();

        match hub
            .events()
            .insert(lesson.into(), calendar_id.as_str())
            .doit()
            .await
        {
            Ok(_) => {}
            Err(_) => {
                println!("Failed to add {}", subject);
                return;
            }
        }

        println!("Added {}", subject);
    }
}

async fn get_lessons(name: String, week: i64) -> Result<Vec<Lesson>> {
    let timetable = class_week::get_timetable(class_week_query::Variables { name, week }).await?;

    let definitions = timetable.schedule_definitions;

    let mut lessons = Vec::new();

    for day in timetable.days {
        for (lesson_index, lesson) in day.lessons.into_iter().enumerate() {
            for details in lesson {
                match parse_details(details, day.date.to_owned(), &definitions[lesson_index]) {
                    Ok(lesson) => lessons.push(lesson),
                    Err(err) => println!("{}", err),
                }
            }
        }
    }

    Ok(lessons)
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
