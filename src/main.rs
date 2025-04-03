use ::reqwest::blocking::Client;
use clap::Parser;
use graphql_client::GraphQLQuery;
use graphql_client::reqwest::post_graphql_blocking as post_graphql;
use owo_colors::{DynColors, OwoColorize};
use std::error::Error;
use unicode_width::UnicodeWidthStr;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct CliArgs {
    #[arg(short, long)]
    user_name: String,

    #[arg(short, long, default_value = " ")]
    repre: String,
}

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

impl DayContribution {
    fn get_month(&self) -> i32 {
        let date = &self.date;
        date.split("-")
            .nth(1)
            .and_then(|month| month.parse().ok())
            .expect("Failed to parse month")
    }
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

fn transpose(contributions: &[Vec<DayContribution>]) -> Vec<Vec<&DayContribution>> {
    let mut rows: Vec<Vec<&DayContribution>> = Vec::with_capacity(7);
    for col in 0..7 {
        let mut new_row: Vec<&DayContribution> = Vec::with_capacity(contributions.len());
        for row in contributions {
            if let Some(day_contribution) = row.get(col) {
                new_row.push(day_contribution);
            }
        }
        rows.push(new_row);
    }

    rows
}

fn draw_heatmap(contributions: &[Vec<&DayContribution>], heatmap_repre: &str) {
    assert!(
        heatmap_repre.width() == 2,
        "heatmap_repre should be width of 2, but width of {} is {}",
        heatmap_repre,
        heatmap_repre.width()
    );

    for row in contributions {
        for day_contribution in row {
            let rgb = day_contribution.color.hex_to_rgb();
            let color = DynColors::Rgb(rgb.0, rgb.1, rgb.2);
            print!("{}", heatmap_repre.color(color));
        }
        println!();
    }
}

fn print_month(contributions: &[Vec<&DayContribution>]) {
    let mut month_line = "".to_string();
    let mut previous_month = 0;
    for (col_idx, _) in contributions[0].iter().enumerate() {
        let mut months_count = [0; 12];
        for row in contributions {
            if let Some(day_contribution) = row.get(col_idx) {
                months_count[(day_contribution.get_month() - 1) as usize] += 1
            }
        }

        let most_appeared_month = months_count
            .iter()
            .enumerate()
            .max_by(|(_, val0), (_, val1)| val0.cmp(val1))
            .map(|(idx, _)| idx)
            .expect("Failed to parse month");

        if most_appeared_month != previous_month {
            month_line.push_str(format!("{:02}", most_appeared_month + 1).as_str());
        } else {
            month_line.push_str("  ");
        }

        previous_month = most_appeared_month;
    }

    println!("{month_line}");
}

fn main() {
    let cli_args = CliArgs::parse();
    let user_name = cli_args.user_name;
    let heatmap_repre = cli_args.repre;
    let response_data = post_graphql_request(user_name).expect("Failed to post GraphQL request");

    let github_status = parse_github_status(response_data).expect("Failed to parse GitHub status");
    let transposed_contributions = transpose(&github_status);
    print_month(&transposed_contributions);
    draw_heatmap(&transposed_contributions, heatmap_repre.as_str());
}
