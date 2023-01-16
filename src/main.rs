use anyhow::{Context, Ok, Result};
use dotenv::dotenv;
use google_calendar3::{oauth2, CalendarHub};
use std::env;

use eacal::{calendar::google_calendar::GoogleCalendar, EACal};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    let service_account_key = oauth2::read_service_account_key("credentials.json")
        .await
        .context("Failed to read credentials.json")?;

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

    let google_calendar = GoogleCalendar::new(hub, calendar_id);

    let eacal = EACal::new(google_calendar);

    eacal.sync("R4C", 23).await?;
    // eacal.sync_current("R4C").await?;

    Ok(())
}
