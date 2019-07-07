#[macro_use]
extern crate diesel;

use actix_web::{web, App, Responder, HttpServer, HttpResponse};
use actix_web::http::header;
use actix_web::middleware::cors;
use actix_web::middleware::Logger;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::sql_query;
use dotenv::dotenv;
use env_logger;
use handlebars::Handlebars;
use listenfd::ListenFd;
use self::models::*;
use serde::Deserialize;
use serde_json::json;
use std::collections::BTreeMap;
use std::env;
use std::error::Error;
use std::str::FromStr;

mod schema;
mod models;

#[derive(Deserialize)]
struct SearchQuery {
    q: Option<String>,
    goal_area_ids: Option<String>,
}

struct AppState {
    pub template_registry: Handlebars,
}

// Registers the Handlebars templates for the application.
fn register_templates() -> Result<Handlebars, Box<dyn Error>> {
    let mut template_registry = Handlebars::new();
    template_registry.set_strict_mode(true);
    template_registry.register_templates_directory(".hbs", "./web/templates/")?;

    Ok(template_registry)
}

pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}

pub fn create_goal_area(conn: &PgConnection, description: &String) -> GoalArea {
    use schema::goal_areas;

    let new_goal_area = NewGoalArea {
        description,
    };

    diesel::insert_into(goal_areas::table)
        .values(&new_goal_area)
        .get_result(conn)
        .expect("Error saving new goal_area")
}

fn index(data: web::Data<AppState>) -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html")
        .body(data.template_registry.render("home", &json!({})).unwrap())
}

fn api_search(query: web::Query<SearchQuery>) -> impl Responder {
    let connection = establish_connection();

    let objective_results = sql_query(
        objective_search_sql(&query.q, &query.goal_area_ids))
        .load::<CategorizedObjective>(&connection)
        .expect("Error loading objectives");

    let result = json!({
      "data": {
        "objectives": &objective_results
      }
    });

    result.to_string()
}

// We find objectives as follows:
// If a search query parameter (q) is given, then we want any objectives that either have
// a description matching that query text, or have tags matching that query text.
// If no search query is given, then it provides no constraint.
// If a goal_area_ids parameter is given, then we want to constrain the loaded objectives
// to ones within those goal areas. If no goal_area_ids is given, then it provides no constraint.
// Both q and goal_area_ids can be given but empty, and we will load no objectives in that case.
fn objective_search_sql(q: &Option<String>, goal_area_ids: &Option<String>) -> String {
    let tag_search_clause = text_search_clause(&String::from("WHERE t.name @@ to_tsquery('{{ts_query_terms}}')"),
                                               q);
    let description_search_clause = text_search_clause(&String::from("objectives.ts_description @@ to_tsquery('{{ts_query_terms}}')"),
                                                       q);

    let search_query_clause = if description_search_clause.len() > 0 && tag_search_clause.len() > 0 {
        format!("({} OR objectives.id = matching_tags.objective_id)", description_search_clause)
    } else {
        String::from("1=1")
    };

    let goal_area_ids_clause = goal_area_ids_clause(&goal_area_ids);

    format!("SELECT objectives.id, objectives.description,
                CASE
                    WHEN containing_goal_areas.goal_area_ids is NULL THEN '{{}}'
                    ELSE containing_goal_areas.goal_area_ids
                END AS goal_area_ids,
                CASE
                    WHEN containing_tags.tag_ids is NULL THEN '{{}}'
                    ELSE containing_tags.tag_ids
                END AS tag_ids
            FROM objectives
            LEFT OUTER JOIN (
                SELECT oga.objective_id, array_agg(oga.goal_area_id) AS goal_area_ids
                FROM objectives_goal_areas oga
                GROUP BY oga.objective_id
            ) containing_goal_areas
            ON objectives.id = containing_goal_areas.objective_id
            LEFT OUTER JOIN (
                SELECT ot.objective_id, array_agg(ot.tag_id) AS tag_ids
                FROM objectives_tags ot
                GROUP BY ot.objective_id
            ) containing_tags
            ON objectives.id = containing_tags.objective_id
            LEFT OUTER JOIN (
                SELECT ot.objective_id
                FROM objectives_tags ot
                JOIN tags t
                ON ot.tag_id = t.id
                {}
                GROUP BY ot.objective_id
            ) matching_tags
            ON objectives.id = matching_tags.objective_id
            WHERE {}
            AND {}",
            tag_search_clause,
            goal_area_ids_clause,
            search_query_clause
            )
}

fn text_search_clause(condition_text: &String, search_query: &Option<String>) -> String {
    let reg = Handlebars::new();

    match &search_query {
        Some(search_phrase) => {
            let mut search_terms = Vec::new();
            // TODO: implement first class query builder support for postgres full text search
            // for now we do primitive string escaping
            for term in str::split_whitespace(&str::replace(search_phrase, "'", "''")) {
                search_terms.push(format!("{}:*", term));
            }
            let context = &json!({ "ts_query_terms": search_terms.join(" | ") });
            reg.render_template(condition_text, context).unwrap()
        },
        None => String::from(""),
    }
}

fn goal_area_ids_clause(goal_area_ids: &Option<String>) -> String {
    match goal_area_ids {
        Some(v) => {
            // we coerce the given goal_area_ids parameter it into integers to ensure
            // that it consists only of values with the same data type as the goal_areas IDs in the DB
            let requested_goal_area_ids : Vec<i32> = v.split(",")
                .map(|id| i32::from_str(id))
                .filter_map(Result::ok)
                .collect();

            let joined_goal_area_ids = requested_goal_area_ids.iter()
                .map(|i| i.to_string())
                .collect::<Vec<String>>()
                .join(",");
            format!("goal_area_ids && ARRAY[{}]::integer[]", joined_goal_area_ids)
        },
        None => String::from("1=1"),
    }
}

fn search(data: web::Data<AppState>) -> HttpResponse {
    use schema::{goal_areas, tags};

    let connection = establish_connection();
    let goal_area_results = goal_areas::table
        .load::<GoalArea>(&connection)
        .expect("Error loading goal_areas");
    let tag_results = tags::table
        .load::<Tag>(&connection)
        .expect("Error loading tags");

    let mut goal_areas = Vec::new();
    for goal_area in goal_area_results {
        let mut item= BTreeMap::new();
        item.insert(String::from("description"), goal_area.description);
        goal_areas.push(item);
    }

    let mut tags = Vec::new();
    for tag in tag_results {
        let mut item= BTreeMap::new();
        item.insert(String::from("name"), tag.name);
        tags.push(item);
    }

    let json = json!({
        "data": {
          "clientName": "Client",
          "goalAreas": &goal_areas,
          "tags": &tags
        }
    });

    let context = json!({
      "appEnvironment": format!("{}", env::var("APP_ENVIRONMENT").unwrap()),
      "baseURL": format!("{}", env::var("BASE_URL").unwrap()),
      "scriptURL": format!("https://objective-bank.s3.amazonaws.com/app-{}.js", env::var("CLIENT_JS_ID").unwrap()),
      "cssURL": format!("https://objective-bank.s3.amazonaws.com/app-{}.css", env::var("CLIENT_CSS_ID").unwrap()),
      "data": json.to_string()
    });

    HttpResponse::Ok()
        .content_type("text/html")
        .body(data.template_registry.render("objectives", &context).unwrap())
}

fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    let mut listenfd = ListenFd::from_env();

    // Get the port number to listen on.
    let port = env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse()
        .expect("PORT must be a number");

    let mut server = HttpServer::new(|| App::new()
        .data(AppState { template_registry: register_templates().unwrap() })
        .wrap(Logger::default())
        .wrap(Logger::new("%a %{User-Agent}i"))
        .wrap(
            cors::Cors::new()
                .allowed_methods(vec!["GET", "POST"])
                .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])
                .allowed_header(header::CONTENT_TYPE)
                .max_age(3600)
        )
        .service(web::resource("/").to(index))
        .service(web::resource("/api/search").to(api_search))
        .service(web::resource("/search").to(search))
    );

    server = if let Some(l) = listenfd.take_tcp_listener(0).unwrap() {
        server.listen(l).unwrap()
    } else {
        server.bind(("0.0.0.0", port)).unwrap()
    };

    server.run()
}