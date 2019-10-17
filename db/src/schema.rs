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
        ts_config_name -> Regconfig,
//        ts_description -> Tsvector,
    }
}

table! {
    objectives_goal_areas (id) {
        id -> Int4,
        objective_id -> Int4,
        goal_area_id -> Int4,
    }
}

table! {
    objectives_tags (id) {
        id -> Int4,
        objective_id -> Int4,
        tag_id -> Int4,
    }
}

table! {
    tags (id) {
        id -> Int4,
        name -> Text,
        ts_config_name -> Regconfig,
//        ts_name -> Tsvector,
    }
}

joinable!(objectives_goal_areas -> goal_areas (goal_area_id));
joinable!(objectives_goal_areas -> objectives (objective_id));
joinable!(objectives_tags -> objectives (objective_id));
joinable!(objectives_tags -> tags (tag_id));

allow_tables_to_appear_in_same_query!(
    goal_areas,
    objectives,
    objectives_goal_areas,
    objectives_tags,
    tags,
);
