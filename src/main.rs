#[macro_use]
extern crate diesel;

use actix_web::{web, App, Responder, HttpServer, HttpResponse};
use actix_web::http::header;
use actix_web::middleware::cors;
use actix_web::middleware::Logger;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use dotenv::dotenv;
use handlebars::Handlebars;
use env_logger;
use listenfd::ListenFd;
use self::models::*;
use serde::{Deserialize};
use serde_json::json;
use std::collections::BTreeMap;
use std::env;
use std::error::Error;

mod schema;
mod models;

#[derive(Deserialize)]
struct DataPayload {
    data: NewGoalAreaPayload,
}

#[derive(Deserialize)]
struct NewGoalAreaPayload {
    description: String,
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

fn api_create_goal_area(payload: web::Json<DataPayload>) -> impl Responder {
    let connection = establish_connection();

    let goal_area = create_goal_area(&connection, &payload.data.description);

    let result = json!({
      "data": { "id": format!("{}", goal_area.id), "description": "An objective" }
    });

    result.to_string()
}

fn api_search() -> impl Responder {
    let result = json!({
      "data": [
        { "description": "An objective" }
      ]
    });

    result.to_string()
}

fn search(data: web::Data<AppState>) -> HttpResponse {
    use schema::goal_areas;

    let connection = establish_connection();
    let results = goal_areas::table
        .load::<GoalArea>(&connection)
        .expect("Error loading goal_areas");

    let mut goal_areas = Vec::new();
    for goal_area in results {
        let mut item= BTreeMap::new();
        item.insert(String::from("description"), goal_area.description);
        goal_areas.push(item);
    }

    let json = json!({
        "data": {
          "clientName": "Client",
          "goalAreas": &goal_areas,
          "tags": [
            { "name": "vocabulary" },
            { "name": "semantic" },
            { "name": "social inferencing" },
            { "name": "perspective taking" }
          ]
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
        .service(web::resource("/api/goal_areas").route(web::post().to(api_create_goal_area)))
    );

    server = if let Some(l) = listenfd.take_tcp_listener(0).unwrap() {
        server.listen(l).unwrap()
    } else {
        server.bind(("0.0.0.0", port)).unwrap()
    };

    server.run()
}