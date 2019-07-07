CREATE TABLE objectives_goal_areas (
    id SERIAL PRIMARY KEY,
    objective_id INTEGER REFERENCES objectives (id) NOT NULL,
    goal_area_id INTEGER REFERENCES goal_areas (id) NOT NULL
);

CREATE INDEX objectives_goal_areas_objective_id_idx ON objectives_goal_areas USING BTREE (objective_id);
CREATE INDEX objectives_goal_areas_goal_area_id_idx ON objectives_goal_areas USING BTREE (goal_area_id);
