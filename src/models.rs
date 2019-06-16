use super::schema::goal_areas;

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

#[derive(Insertable)]
#[table_name="goal_areas"]
pub struct NewGoalArea<'a> {
    pub description: &'a str,
}