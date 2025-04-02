use ::reqwest::blocking::Client;
use std::error::Error;
// use anyhow::Result;
use graphql_client::GraphQLQuery;
use graphql_client::reqwest::post_graphql_blocking as post_graphql;

type Date = String;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "query.graphql",
    response_derives = "Debug"
)]
struct HeatmapQuery;

struct DayContribution {
    date: String,
    color: String,
}

fn parse_github_status(response_data: heatmap_query::ResponseData) -> Result<Vec<Vec<DayContribution>>, String> {
    match response_data.user {
        Some(user) => {
            let contribution_calendar = user.contributions_collection.contribution_calendar;
            // println!("contribution calendar: {:#?}", contribution_calendar);
            let week_status: Vec<Vec<DayContribution>> = contribution_calendar
                .weeks
                .iter()
                .map(|week_data| {
                    week_data
                        .contribution_days
                        .iter()
                        .map(|day_data| DayContribution {
                            date: day_data.date.clone(),
                            color: day_data.color.clone(),
                        })
                        .collect()
                })
                .collect();

            Ok(week_status)
        }
        None => {
            let err_msg = "User not found";
            println!("{err_msg}");
            Err(String::from(err_msg))
        }
    }
}

fn post_graphql_request(user_name: String) -> Result<heatmap_query::ResponseData, Box<dyn Error>> {
    let github_api_token =
        std::env::var("GITHUB_API_TOKEN").expect("Failed to get GITHUB_API_TOKEN variable");
    let query_variables = heatmap_query::Variables { user_name };
    let client = Client::builder()
        .user_agent("graphql-client/0.10.0")
        .default_headers(
            std::iter::once((
                reqwest::header::AUTHORIZATION,
                reqwest::header::HeaderValue::from_str(&format!("Bearer {}", github_api_token))
                    .unwrap(),
            ))
            .collect(),
        )
        .build()?;

    let response_body = post_graphql::<HeatmapQuery, _>(
        &client,
        "https://api.github.com/graphql",
        query_variables,
    )?;

    let response_data = response_body.data.expect("Missing response data");

    Ok(response_data)
}

fn transpose(contributions: &[DayContribution]) -> Vec<&DayContribution> {
    let mut transposed_contributions: Vec<&DayContribution> =
        Vec::with_capacity(contributions.len());

    let total_rows = contributions.len().div_ceil(7);
    for row_idx in 0..total_rows {
        for col_idx in 0..7 {
            let idx = row_idx * 7 + col_idx;
            // Is this logic necessary if using contributions.get?
            if idx < contributions.len() {
                if let Some(contribution) = contributions.get(idx) {
                    // transposed_contributions
                }
            }
        }
    }

    transposed_contributions
}

fn draw_heatmap(contributions: &[DayContribution]) {}

fn main() {
    let user_name = String::from("xtayex");
    let response_data = post_graphql_request(user_name).expect("Failed to post GraphQL request");

    parse_github_status(response_data);
    // match github_status {
    //     Ok(response_data) => {
    //         let transposed_contributions = transpose(&response_data);
    //     }
    //     Err(err_msg) => {
    //         println!("{err_msg}");
    //         std::process::exit(1);
    //     }
    // }
}
