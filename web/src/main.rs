use actix_web::{web, App, Responder, HttpServer, HttpResponse};
use actix_web::http::header;
use actix_web::middleware::cors;
use actix_web::middleware::Logger;
use db::models::*;
use db::{establish_connection, objective_search_sql};
use diesel::prelude::*;
use diesel::sql_query;
use dotenv::dotenv;
use env_logger;
use handlebars::Handlebars;
use listenfd::ListenFd;
use serde::Deserialize;
use serde_json::json;
use std::collections::BTreeMap;
use std::env;
use std::error::Error;

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

fn search(data: web::Data<AppState>) -> HttpResponse {
    use db::schema::{goal_areas, tags};

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
    dotenv().ok();
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
