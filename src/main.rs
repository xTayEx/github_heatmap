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

fn parse_github_status(
    response_data: heatmap_query::ResponseData,
) -> Result<Vec<DayContribution>, String> {
    match response_data.user {
        Some(user) => {
            let mut contributions: Vec<DayContribution> = Vec::new();
            let contribution_calendar = user.contributions_collection.contribution_calendar;
            // println!("contribution calendar: {:#?}", contribution_calendar);
            let weeks_data = contribution_calendar.weeks;
            for week in weeks_data {
                let contribution_days = week.contribution_days;
                for day in contribution_days {
                    println!("date: {:?}, color: {:?}", day.date, day.color);
                    contributions.push(DayContribution {
                        date: day.date,
                        color: day.color,
                    });
                }
            }
            Ok(contributions)
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

    // let total_rows = (contributions.len() + 7 - 1) / 7;
    let total_rows = contributions.len().div_ceil(7);
    for row_idx in 0..total_rows {
        for col_idx in 0..7 {
            let idx = row_idx + col_idx * total_rows;
            if idx < contributions.len() {
                if let Some(contribution) = contributions.get(idx) {
                    transposed_contributions.push(contribution);
                }
            }
        }
    }

    transposed_contributions
}

fn draw_heatmap(contributions: &[DayContribution]) {

}

fn main() {
    let user_name = String::from("xtayex");
    let response_data = post_graphql_request(user_name).expect("Failed to post GraphQL request");

    let github_status = parse_github_status(response_data);
    match github_status {
        Ok(response_data) => {
            let transposed_contributions = transpose(&response_data);
        }
        Err(err_msg) => {
            println!("{err_msg}");
            std::process::exit(1);
        }
    }
}
