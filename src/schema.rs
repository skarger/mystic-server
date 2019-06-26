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
    }
}

table! {
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
