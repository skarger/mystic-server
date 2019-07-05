table! {
    goal_areas (id) {
        id -> Int4,
        description -> Text,
    }
}

table! {
    objectives (id) {
        id -> Int4,
        description -> Text,
//        ts_config_name -> Regconfig,
//        ts_description -> Tsvector,
    }
}

table! {
    tags (id) {
        id -> Int4,
        name -> Text,
//        ts_config_name -> Regconfig,
//        ts_name -> Tsvector,
    }
}

allow_tables_to_appear_in_same_query!(
    goal_areas,
    objectives,
    tags,
);
