extern crate juniper;

use juniper::{FieldResult, Variables};
use juniper::http::graphiql::graphiql_source;

pub fn execute() -> juniper::Value {
    // Create a context object.
    let ctx = Context {};

    // Run the executor.
    let (res, _errors) = juniper::execute(
        "query { human(id: \"1\") { id } }",
        None,
        &Schema::new(Query {}, Mutation {}),
        &Variables::new(),
        &ctx,
    ).unwrap();

    res
}

pub fn graphiql_html(graphql_url: &String) -> String {
    graphiql_source(graphql_url)
}

#[derive(juniper::GraphQLEnum)]
enum Episode {
    NewHope,
    Empire,
    Jedi,
}

#[derive(juniper::GraphQLObject)]
#[graphql(description="A humanoid creature in the Star Wars universe")]
struct Human {
    id: String,
    name: String,
    appears_in: Vec<Episode>,
    home_planet: String,
}

// There is also a custom derive for mapping GraphQL input objects.

#[derive(juniper::GraphQLInputObject)]
#[graphql(description="A humanoid creature in the Star Wars universe")]
struct NewHuman {
    name: String,
    appears_in: Vec<Episode>,
    home_planet: String,
}

// Now, we create our root Query and Mutation types with resolvers by using the
// object macro.
// Objects can have contexts that allow accessing shared state like a database
// pool.

struct Context {
    // Use your real database pool here.
    // pool: DatabasePool,
}

// To make our context usable by Juniper, we have to implement a marker trait.
impl juniper::Context for Context {}

struct Query {}

#[juniper::object(
// Here we specify the context type for the object.
// We need to do this in every type that
// needs access to the context.
Context = Context,
)]
impl Query {
    fn apiVersion() -> &str {
        "1.0"
    }

    fn human(context: &Context, id: String) -> FieldResult<Human> {
        // Get a db connection.
        //let connection = context.pool.get_connection()?;
        // Execute a db query.
        // Note the use of `?` to propagate errors.
        //let human = connection.find_human(&id)?;
        let human = Human {
            id,
            name: "name".to_string(),
            appears_in: vec![Episode::NewHope],
            home_planet: "p".to_string()
        };
        // Return the result.
        Ok(human)
    }
}
//});


struct Mutation {}

#[juniper::object(
Context = Context,
)]
impl Mutation {}

// A root schema consists of a query and a mutation.
// Request queries can be executed against a RootNode.
type Schema = juniper::RootNode<'static, Query, Mutation>;