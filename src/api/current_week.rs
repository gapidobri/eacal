use anyhow::{Context, Result};
use graphql_client::{GraphQLQuery, Response};

use self::current_week_query::{ResponseData, Variables};

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/current_week.graphql",
    response_derives = "Debug"
)]
pub struct CurrentWeekQuery;

pub async fn get_current_week() -> Result<u8> {
    let request_body = CurrentWeekQuery::build_query(Variables {});

    let client = reqwest::Client::new();

    let res = client
        .post("https://v3.vegova.sync.si/graphql")
        .json(&request_body)
        .send()
        .await?;

    let response_body: Response<ResponseData> = res.json().await?;

    let current_week = response_body
        .data
        .context("Failed to get current week")?
        .current_week
        .try_into()?;

    Ok(current_week)
}
