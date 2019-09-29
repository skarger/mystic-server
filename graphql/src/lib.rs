use juniper;

use juniper::{FieldResult, EmptyMutation};
use juniper::http::graphiql::graphiql_source;
pub use juniper::http::GraphQLRequest;

use db::{ConnectionPool, load_goal_areas, load_tags, search_for_objectives};

// A root schema consists of a query and a mutation.
// Request queries can be executed against a RootNode.
pub type Schema = juniper::RootNode<'static, Query, EmptyMutation<Context>>;

pub struct Context {
    pub connection_pool: ConnectionPool
}

// To make our context usable by Juniper, we have to implement a marker trait.
impl juniper::Context for Context {}

pub struct Query {}

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

    fn objectives(context: &Context, filter: ObjectiveFilterInput) -> FieldResult<Vec<CategorizedObjectiveType>> {
        let connection = context.connection_pool.get().unwrap();
        let objectives = search_for_objectives(&connection, &filter.q, &filter.goal_area_ids);
        let result = objectives
            .into_iter()
            .map(|obj|
                CategorizedObjectiveType {
                    id: obj.id,
                    description: obj.description,
                    goal_area_ids: obj.goal_area_ids,
                    tag_ids: obj.tag_ids,
                }
            ).collect();
        Ok(result)
    }
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

#[derive(juniper::GraphQLInputObject)]
struct ObjectiveFilterInput {
    pub q: Option<String>,
    pub goal_area_ids: Option<Vec<i32>>,
}

#[derive(juniper::GraphQLObject)]
pub struct CategorizedObjectiveType {
    pub id: i32,
    pub description: String,
    #[graphql(name="goalAreaIds")]
    pub goal_area_ids: Vec<i32>,
    #[graphql(name="tagIds")]
    pub tag_ids: Vec<i32>,
}

pub fn create_schema() -> Schema {
    Schema::new(Query {}, EmptyMutation::new())
}
pub fn graphiql_html(graphql_url: &String) -> String {
    graphiql_source(graphql_url)
}
