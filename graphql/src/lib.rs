pub use juniper;

use juniper::{FieldResult, EmptyMutation};
use juniper::http::graphiql::graphiql_source;
pub use juniper::http::GraphQLRequest;

use db::{ConnectionPool, load_goal_areas, load_tags};

// A root schema consists of a query and a mutation.
// Request queries can be executed against a RootNode.
pub type Schema = juniper::RootNode<'static, Query, EmptyMutation<Context>>;

pub struct Context {
    pub connection_pool: ConnectionPool
}
pub struct Query {}

pub fn create_schema() -> Schema {
    Schema::new(Query {}, EmptyMutation::new())
}

pub fn graphiql_html(graphql_url: &String) -> String {
    graphiql_source(graphql_url)
}

#[derive(juniper::GraphQLObject)]
struct GoalAreaType {
    pub id: i32,
    pub description: String,
}

#[derive(juniper::GraphQLObject)]
struct TagType {
    pub id: i32,
    pub name: String,
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





// To make our context usable by Juniper, we have to implement a marker trait.
impl juniper::Context for Context {}

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

    fn goal_areas(context: &Context) -> FieldResult<Vec<GoalAreaType>> {
        let connection = context.connection_pool.get().unwrap();
        let goal_areas = load_goal_areas(&connection);
        let result = goal_areas
            .into_iter()
            .map(|ga| GoalAreaType { id: ga.id, description: ga.description })
            .collect();
        Ok(result)
    }

    fn tags(context: &Context) -> FieldResult<Vec<TagType>> {
        let connection = context.connection_pool.get().unwrap();
        let tags = load_tags(&connection);
        let result = tags
            .into_iter()
            .map(|tag| TagType { id: tag.id, name: tag.name })
            .collect();
        Ok(result)
    }
}
