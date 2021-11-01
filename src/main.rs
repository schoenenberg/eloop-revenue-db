#![warn(unsafe_code)]

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

use clap::{crate_authors, crate_description, crate_name, crate_version};
use diesel::{Connection, QueryDsl, RunQueryDsl};

mod dto;
mod models;
mod schema;

#[derive(Debug)]
struct ParsedProfit {
    net_profit: Vec<dto::ProfitDay>,
    revenue: Vec<dto::ProfitDay>,
}

impl From<dto::PersonalResponse> for ParsedProfit {
    fn from(resp: dto::PersonalResponse) -> Self {
        let mut revenue: Vec<dto::ProfitDay> = Vec::new();
        let mut net_profit: Vec<dto::ProfitDay> = Vec::new();

        for (k, v) in resp.result {
            let (year_str, month_str) = k.split_once("-").expect("Should be a date");

            for (day_str, value) in &v.net_profit.data {
                let year = year_str.parse().expect("Year not parseable");
                let month = month_str.parse().expect("Month not parseable");
                let day = day_str.parse().expect("Day not parseable");

                let date = chrono::NaiveDate::from_ymd(year, month, day);
                net_profit.push(dto::ProfitDay {
                    date,
                    value: value.to_owned(),
                });
            }
            for (day_str, value) in &v.revenue.data {
                let year = year_str.parse().expect("Year not parseable");
                let month = month_str.parse().expect("Month not parseable");
                let day = day_str.parse().expect("Day not parseable");

                let date = chrono::NaiveDate::from_ymd(year, month, day);
                revenue.push(dto::ProfitDay {
                    date,
                    value: value.to_owned(),
                });
            }
        }
        Self {
            net_profit,
            revenue,
        }
    }
}

embed_migrations!();

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let matches = clap::app_from_crate!()
        .subcommand(clap::SubCommand::with_name("update"))
        .subcommand(clap::SubCommand::with_name("summary"))
        .subcommand(
            clap::SubCommand::with_name("config")
                .arg(
                    clap::Arg::with_name("rest-api-key")
                        .long("rest-api-key")
                        .takes_value(true),
                )
                .arg(
                    clap::Arg::with_name("session-token")
                        .long("session-token")
                        .takes_value(true),
                ),
        )
        .get_matches();

    // Handle path to db
    let db_url = if let Ok(u) = std::env::var("ELOOP_DATABASE_URL") {
        std::path::PathBuf::from(u)
    } else {
        let mut home_dir = dirs::home_dir().expect("$HOME not set. Use ELOOP_DATABASE_URL to omit this behaviour.");
        home_dir.push(".eloop.sqlite");
        home_dir
    };

    // Check if db exists..
    if !db_url.exists() {
        // Create the file
        std::fs::write(&db_url, "")?;
    }

    // Open the database
    log::debug!("Opening database: {}", db_url.as_path().to_str().expect("Just show the damn path"));
    let conn = diesel::sqlite::SqliteConnection::establish(db_url.as_path().to_str().expect("It's just a path.."))?;

    // Run migrations
    embedded_migrations::run(&conn)?;

    match matches.subcommand() {
        ("update", _) => {
            let session_token = load_config_key(&conn, "session-token")?;
            let rest_api_key = load_config_key(&conn, "api-key")?;

            let client = reqwest::blocking::Client::new();
            let resp: dto::PersonalResponse = client
                .post("https://api.eloop.one/functions/getdailystats")
                .header("x-parse-application-id", "eloopapp")
                .header("x-parse-session-token", session_token)
                .header("x-parse-rest-api-key", rest_api_key)
                .send()?
                .json()?;

            let profit_list = ParsedProfit::from(resp);
            let _ = insert_values(&conn, profit_list)?;
            println!("Update successful.")
        }
        ("summary", _) => {
            let personal_revenue = summary(&conn)?;

            if let Some(revenue) = personal_revenue {
                println!("Revenue: € {:.2}", revenue);
            }
        }
        ("config", Some(matches)) => {
            if let Some(api_key) = matches.value_of("rest-api-key") {
                let _ = save_config_key(&conn, "api-key".to_string(), api_key.to_string())?;
            }
            if let Some(session_token) = matches.value_of("session-token") {
                let _ = save_config_key(&conn, "session-token".to_string(), session_token.to_string())?;
            }
        }
        _ => {
            println!("{}", matches.usage());
        }
    };

    Ok(())
}

fn insert_values(
    conn: &diesel::sqlite::SqliteConnection,
    profit: ParsedProfit,
) -> Result<(), Box<dyn std::error::Error>> {
    use self::schema::{net_profits::dsl::*, revenues::dsl::*};

    profit.revenue.iter().for_each(|v| {
        let p = models::Revenue {
            day: v.date,
            value: v.value,
        };
        let _ = diesel::insert_or_ignore_into(revenues)
            .values(p)
            .execute(conn);
    });

    profit.net_profit.iter().for_each(|v| {
        let p = models::NetProfit {
            day: v.date,
            value: v.value,
        };
        let _ = diesel::insert_or_ignore_into(net_profits)
            .values(p)
            .execute(conn);
    });

    Ok(())
}

fn summary(
    conn: &diesel::sqlite::SqliteConnection,
) -> Result<Option<f32>, Box<dyn std::error::Error>> {
    use self::schema::revenues::dsl::*;
    use diesel::dsl::sum;

    Ok(revenues.select(sum(value)).first(conn)?)
}

fn load_config_key(
    conn: &diesel::sqlite::SqliteConnection,
    key_str: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    use self::schema::configs::dsl::*;
    use crate::diesel::ExpressionMethods;

    Ok(configs.select(value).filter(key.eq(key_str)).first(conn)?)
}

fn save_config_key(
    conn: &diesel::sqlite::SqliteConnection,
    key_str: String,
    value_str: String,
) -> Result<(), Box<dyn std::error::Error>> {
    use self::schema::configs::dsl::*;
    use crate::diesel::ExpressionMethods;

    let inserted_rows = diesel::insert_or_ignore_into(configs)
        .values(models::Config { key: key_str.clone(), value: value_str.clone() })
        .execute(conn)?;

    if inserted_rows == 0 {
        let target = configs.filter(key.eq(key_str.clone()));
        let updated_rows = diesel::update(target).set(value.eq(value_str)).execute(conn)?;
        if updated_rows == 0 {
            println!("Failed update of ‘{}‘", key_str);
        }
    }

    Ok(())
}