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