use super::types;

table! {
    use diesel::sql_types::*;

    goal_areas (id) {
        id -> Int4,
        description -> Text,
    }
}

table! {
    use diesel::sql_types::*;
    use super::types;

    objectives (id) {
        id -> Int4,
        description -> Text,
        ts_config_name -> types::RegConfig,
    }
}

table! {
    use diesel::sql_types::*;

    tags (id) {
        id -> Int4,
        name -> Text,
    }
}

allow_tables_to_appear_in_same_query!(
    goal_areas,
    objectives,
    tags,
);
