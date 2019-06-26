use super::schema::goal_areas;
use super::full_text_search::types::*;

#[derive(Queryable)]
pub struct GoalArea {
    pub id: i32,
    pub description: String,
}

#[derive(Queryable)]
pub struct Tag {
    pub id: i32,
    pub name: String,
}

#[derive(Queryable)]
pub struct Objective {
    pub id: i32,
    pub description: String,
    pub ts_config_name: RegConfig,
}

#[derive(Insertable)]
#[table_name="goal_areas"]
pub struct NewGoalArea<'a> {
    pub description: &'a str,
}