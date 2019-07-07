#[macro_use]
extern crate diesel;

use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::sql_query;
use dotenv::dotenv;
use handlebars::Handlebars;
use self::models::*;
use self::schema::*;
use serde_json::json;
use std::env;
use std::str::FromStr;

pub mod schema;
pub mod models;

pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}

pub fn load_goal_areas(connection: &PgConnection) -> Vec<GoalArea> {
    goal_areas::table
        .load::<GoalArea>(connection)
        .expect("Error loading goal_areas")
}

pub fn load_tags(connection: &PgConnection) -> Vec<Tag> {
    tags::table
        .load::<Tag>(connection)
        .expect("Error loading tags")
}

pub fn search_for_objectives(connection: &PgConnection, q: &Option<String>, goal_area_ids: &Option<String>) -> Vec<CategorizedObjective> {
    sql_query(objective_search_sql(q, goal_area_ids))
        .load::<CategorizedObjective>(connection)
        .expect("Error loading objectives")
}

// We find objectives as follows:
// If a search query parameter (q) is Some value, then we constrain the loaded objectives to ones that
// either have a description matching that query text, or have tags matching that query text.
// If a goal_area_ids parameter is Some value, then we constrain the loaded objectives
// to ones within those goal areas.
// Both q and goal_area_ids can be Some value but empty, and we will load no objectives in that case.
// When a parameter is None, that parameter will not constrain the loaded objectives.
fn objective_search_sql(q: &Option<String>, goal_area_ids: &Option<String>) -> String {
    let tag_search_clause = text_search_clause(&String::from("WHERE t.name @@ to_tsquery('{{ts_query_terms}}')"),
                                               q);
    let description_search_clause = text_search_clause(&String::from("objectives.ts_description @@ to_tsquery('{{ts_query_terms}}')"),
                                                       q);

    let search_query_clause = if description_search_clause.len() > 0 && tag_search_clause.len() > 0 {
        format!("({} OR objectives.id = matching_tags.objective_id)", description_search_clause)
    } else {
        String::from("1=1")
    };

    let goal_area_ids_clause = goal_area_ids_clause(&goal_area_ids);

    format!("SELECT objectives.id, objectives.description,
                CASE
                    WHEN containing_goal_areas.goal_area_ids is NULL THEN '{{}}'
                    ELSE containing_goal_areas.goal_area_ids
                END AS goal_area_ids,
                CASE
                    WHEN containing_tags.tag_ids is NULL THEN '{{}}'
                    ELSE containing_tags.tag_ids
                END AS tag_ids
            FROM objectives
            LEFT OUTER JOIN (
                SELECT oga.objective_id, array_agg(oga.goal_area_id) AS goal_area_ids
                FROM objectives_goal_areas oga
                GROUP BY oga.objective_id
            ) containing_goal_areas
            ON objectives.id = containing_goal_areas.objective_id
            LEFT OUTER JOIN (
                SELECT ot.objective_id, array_agg(ot.tag_id) AS tag_ids
                FROM objectives_tags ot
                GROUP BY ot.objective_id
            ) containing_tags
            ON objectives.id = containing_tags.objective_id
            LEFT OUTER JOIN (
                SELECT ot.objective_id
                FROM objectives_tags ot
                JOIN tags t
                ON ot.tag_id = t.id
                {}
                GROUP BY ot.objective_id
            ) matching_tags
            ON objectives.id = matching_tags.objective_id
            WHERE {}
            AND {}",
            tag_search_clause,
            goal_area_ids_clause,
            search_query_clause
    )
}

fn text_search_clause(condition_text: &String, search_query: &Option<String>) -> String {
    let reg = Handlebars::new();

    match &search_query {
        Some(search_phrase) => {
            let mut search_terms = Vec::new();
            // TODO: implement first class query builder support for postgres full text search
            // for now we do primitive string escaping
            for term in str::split_whitespace(&str::replace(search_phrase, "'", "''")) {
                search_terms.push(format!("{}:*", term));
            }
            let context = &json!({ "ts_query_terms": search_terms.join(" | ") });
            reg.render_template(condition_text, context).unwrap()
        },
        None => String::from(""),
    }
}

fn goal_area_ids_clause(goal_area_ids: &Option<String>) -> String {
    match goal_area_ids {
        Some(v) => {
            // we coerce the given goal_area_ids parameter it into integers to ensure
            // that it consists only of values with the same data type as the goal_areas IDs in the DB
            let requested_goal_area_ids : Vec<i32> = v.split(",")
                .map(|id| i32::from_str(id))
                .filter_map(Result::ok)
                .collect();

            let joined_goal_area_ids = requested_goal_area_ids.iter()
                .map(|i| i.to_string())
                .collect::<Vec<String>>()
                .join(",");
            format!("goal_area_ids && ARRAY[{}]::integer[]", joined_goal_area_ids)
        },
        None => String::from("1=1"),
    }
}