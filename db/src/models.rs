use diesel::sql_types::*;
use serde::Serialize;

#[derive(Queryable, Serialize)]
pub struct GoalArea {
    pub id: i32,
    pub description: String,
}

#[derive(Queryable, Serialize)]
pub struct Tag {
    pub id: i32,
    pub name: String,
}

#[derive(QueryableByName, PartialEq, Debug, Serialize)]
pub struct CategorizedObjective {
    #[sql_type="Integer"]
    pub id: i32,
    #[sql_type="Text"]
    pub description: String,
    #[sql_type="Array<Integer>"]
    pub goal_area_ids: Vec<i32>,
    #[sql_type="Array<Integer>"]
    pub tag_ids: Vec<i32>,

}
