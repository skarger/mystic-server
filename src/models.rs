use super::schema::goal_areas;
use super::schema::objectives;

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

#[derive(QueryableByName, PartialEq, Debug)]
#[table_name = "objectives"]
pub struct Objective {
    pub id: i32,
    pub description: String,
}

#[derive(Insertable)]
#[table_name="goal_areas"]
pub struct NewGoalArea<'a> {
    pub description: &'a str,
}