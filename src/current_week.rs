use graphql_client::GraphQLQuery;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/current_week.graphql",
    response_derives = "Debug"
)]
pub struct CurrentWeekQuery;

async fn get_current_week() -> Result<u32, ()> {
    Ok(0)
}
