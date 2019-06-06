use actix_web::{web, App, Responder, HttpServer, HttpResponse};
use actix_web::http::header;
use actix_web::middleware::cors;
use actix_web::middleware::Logger;
use handlebars::Handlebars;
use env_logger;
use listenfd::ListenFd;
use serde_json::json;
use std::collections::{BTreeMap};
use std::env;
use std::fs;


fn index() -> impl Responder {
    let mut handlebars = Handlebars::new();

    // register the template. The template string will be verified and compiled.
    let source = "hello {{world}}";
    assert!(handlebars.register_template_string("index", source).is_ok());

    let mut data = BTreeMap::new();
    data.insert("world".to_string(), "世界!".to_string());
    handlebars.render("index", &data).unwrap()
}

fn api_search() -> impl Responder {
    let result = json!({
      "data": [
        { "description": "An objective" }
      ]
    });

    result.to_string()
}

fn search() -> HttpResponse {
    let data = json!({
        "data": {
            "clientName": "Student",
            "objectives": [
                { "description": "An objective" }
            ]
        }
    });

    let env = json!({
      "scriptURL": "https://objective-bank.s3.amazonaws.com/app-bbf7cb9a309d9faa940a1cfbfb7de87e.js",
      "cssURL": "https://objective-bank.s3.amazonaws.com/app-6314a9ef95b155b4de563d349944e6f3.css",
      "data": data.to_string()
    });

    let mut handlebars = Handlebars::new();
    let filename = "./templates/objectives.hbs";
    let source = fs::read_to_string(filename)
        .expect("Something went wrong reading the file");

    assert!(handlebars.register_template_string("objectives", source).is_ok());

    HttpResponse::Ok()
        .content_type("text/html")
        .body(handlebars.render("objectives", &env).unwrap())
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