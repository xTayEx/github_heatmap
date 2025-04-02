use ::reqwest::blocking::Client;
use std::error::Error;
use owo_colors::{OwoColorize, DynColors};
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

trait HexToRgb {
    fn hex_to_rgb(&self) -> (u8, u8, u8);
}

impl HexToRgb for str {
    fn hex_to_rgb(&self) -> (u8, u8, u8) {
        let hex = self.trim_start_matches('#');
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap();
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap();
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap();
        (r, g, b)
    }
}

fn parse_github_status(
    response_data: heatmap_query::ResponseData,
) -> Result<Vec<Vec<DayContribution>>, String> {
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

fn transpose(contributions: &[Vec<DayContribution>]) -> Vec<Vec<DayContribution>> {
    let mut rows: Vec<Vec<DayContribution>> = Vec::with_capacity(7);
    for col  in 0..7 {
        let mut new_row: Vec<DayContribution> = Vec::with_capacity(contributions.len());
        for row in contributions {
            if let Some(day_contribution) = row.get(col) {
                new_row.push(DayContribution {
                    date: day_contribution.date.clone(),
                    color: day_contribution.color.clone(),
                });
            }
        }
        rows.push(new_row);
    }

    rows
}

fn draw_heatmap(contributions: &[Vec<DayContribution>]) {
    for row in contributions {
        for day_contribution in row {
            let rgb = day_contribution.color.hex_to_rgb();
            let color = DynColors::Rgb(rgb.0, rgb.1, rgb.2);
            print!("{}", " ".color(color));
        }
        println!();
    }
}

fn main() {
    let user_name = String::from("xtayex");
    let response_data = post_graphql_request(user_name).expect("Failed to post GraphQL request");

    let github_status = parse_github_status(response_data).expect("Failed to parse GitHub status");
    let transposed_contributions = transpose(&github_status);
    draw_heatmap(&transposed_contributions);
}
