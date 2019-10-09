use actix_cors;
use actix_web::{web, App, Responder, HttpServer, HttpResponse};
use actix_web::http::header;
use actix_web::middleware::Logger;
use db::{connection_pool, establish_connection, search_for_objectives};
use graphql::{Schema, GraphQLRequest, Context, create_schema, graphiql_html};
use dotenv::dotenv;
use env_logger;
use handlebars::Handlebars;
use listenfd::ListenFd;
use serde::Deserialize;
use serde_json::json;
use std::env;
use std::error::Error;
use std::str::FromStr;
use std::sync::Arc;

#[derive(Deserialize)]
struct SearchQuery {
    q: Option<String>,
    goal_area_ids: Option<String>,
}

struct AppState {
    pub template_registry: Handlebars,
    pub graphql_schema: Arc<Schema>,
    pub graphql_context: Context,
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
    let goal_area_ids = match &query.goal_area_ids {
        None => { None }
        Some(v) => {
            // we coerce the given goal_area_ids parameter into integers to ensure
            // that it consists only of values with the same data type as the goal_areas IDs in the DB
            let requested_goal_area_ids: Vec<i32> = v.split(",")
                .map(|id| i32::from_str(id))
                .filter_map(Result::ok)
                .collect();

            Some(requested_goal_area_ids)
        }
    };

    let objectives = search_for_objectives(&connection, &query.q, &goal_area_ids);

    let result = json!({
      "data": {
        "objectives": &objectives
      }
    });

    result.to_string()
}

fn search(data: web::Data<AppState>) -> HttpResponse {
    let context = json!({
      "appEnvironment": format!("{}", env::var("APP_ENVIRONMENT").unwrap()),
      "baseURL": format!("{}", env::var("BASE_URL").unwrap()),
      "scriptURL": format!("{}/app-{}.js", env::var("CLIENT_BASE_URL").unwrap(), env::var("CLIENT_JS_ID").unwrap()),
    });

    HttpResponse::Ok()
        .content_type("text/html")
        .body(data.template_registry.render("search", &context).unwrap())
}

fn graphql(data: web::Data<AppState>, graphql_request: web::Json<GraphQLRequest>,) -> HttpResponse {
    let res = graphql_request.execute(&data.graphql_schema, &data.graphql_context);

    HttpResponse::Ok()
        .header(header::CONTENT_TYPE, "application/json")
        .body(serde_json::to_string(&res).unwrap())
}

fn graphiql() -> HttpResponse {
    let graphql_url = format!("{}/graphql", env::var("BASE_URL").unwrap());
    let html = graphiql_html(&graphql_url);
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}

fn main() -> std::io::Result<()> {
    dotenv().ok();
    std::env::set_var("RUST_LOG", "actix_web=info,web=info");
    env_logger::init();

    let mut listenfd = ListenFd::from_env();

    // Get the port number to listen on.
    let port = env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse()
        .expect("PORT must be a number");

    let graphql_schema = std::sync::Arc::new(create_schema());

    let mut server = HttpServer::new(move || {
        App::new()
            .data(AppState {
                template_registry: register_templates().unwrap(),
                graphql_schema: Arc::clone(&graphql_schema),
                graphql_context: Context { connection_pool: connection_pool() },
            })
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"))
            .wrap(
                actix_cors::Cors::new()
                    .allowed_methods(vec!["GET", "POST"])
                    .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])
                    .allowed_header(header::CONTENT_TYPE)
                    .max_age(3600)
            )
            .service(web::resource("/").to(index))
            .service(web::resource("/api/search").to(api_search))
            .service(web::resource("/search").to(search))
            .service(web::resource("/graphql").route(web::post().to(graphql)))
            .service(web::resource("/graphiql").route(web::get().to(graphiql)))
    });

    server = if let Some(l) = listenfd.take_tcp_listener(0).unwrap() {
        server.listen(l).unwrap()
    } else {
        server.bind(("0.0.0.0", port)).unwrap()
    };

    server.run()
}
