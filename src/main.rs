use actix_web::{web, App, Responder, HttpServer};
use std::env;

fn index(info: web::Path<(String, u32)>) -> impl Responder {
    format!("Hello {}! id:{}", info.0, info.1)
}

fn main() -> std::io::Result<()> {
    // Get the port number to listen on.
    let port = env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse()
        .expect("PORT must be a number");

    HttpServer::new(|| App::new().service(
        web::resource("/{name}/{id}/index.html").to(index))
    )
        .bind(("0.0.0.0", port))?
        .run()
}