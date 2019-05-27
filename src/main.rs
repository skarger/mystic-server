use actix_web::{web, App, Responder, HttpResponse, HttpServer};
use std::{env};
use juniper::{FieldResult, Variables, http::GraphQLRequest};
use juniper::http::graphiql::graphiql_source;
use serde_json;

#[derive(juniper::GraphQLObject)]
#[graphql(description="An organizing label applied to an objective, represented as a string")]
struct Tag {
    external_id: String,
}

#[derive(juniper::GraphQLObject)]
#[graphql(description="A therapy objective")]
struct Objective {
    description: String,
    tags: Vec<Tag>,
}

struct Query;

#[juniper::object()]
impl Query {
    fn apiVersion() -> &str {
        "1.0"
    }

    fn objectives() -> FieldResult<Vec<Objective>> {

        let tags1 = vec![Tag { external_id: "Social Communication".to_string() }];
        let tags2 = vec![Tag { external_id: "Expressive Morphology".to_string() }];

        let objective1 = Objective { description: String::from("%1$s will sustain a reciprocal conversation for three turns."), tags: tags1 };
        let objective2 = Objective { description: String::from("In structured activities, %1$s will use question form \"is it <blank>]\" given intermittent verbal models."), tags: tags2 };

        Ok(vec![objective1, objective2])
    }
}

struct Mutation;

#[juniper::object()]
impl Mutation {
}

// A root schema consists of a query and a mutation.
// Request queries can be executed against a RootNode.
type Schema = juniper::RootNode<'static, Query, Mutation>;

fn graphql(
    _data: web::Json<GraphQLRequest>,
) -> impl Responder {
    let (res, _errors) = juniper::execute(
        "query { objectives { description tags { externalId } } }",
        None,
        &Schema::new(Query, Mutation),
        &Variables::new(),
        &(),
    ).unwrap();

    serde_json::to_string(&res)
}


fn graphiql() -> HttpResponse {
    let html = graphiql_source("/graphql");
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}

fn index() -> impl Responder {
    format!("Hello!")
}

fn main() -> std::io::Result<()> {
    // Get the port number to listen on.
    let port = env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse()
        .expect("PORT must be a number");

    HttpServer::new(|| App::new()
        .service(web::resource("/").to(index))
        .service(web::resource("/graphql").route(web::post().to(graphql)))
        .service(web::resource("/graphiql").route(web::get().to(graphiql))))
        .bind(("0.0.0.0", port))?
        .run()
}