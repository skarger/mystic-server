CREATE TABLE objectives_tags (
    id SERIAL PRIMARY KEY,
    objective_id INTEGER REFERENCES objectives (id) NOT NULL,
    tag_id INTEGER REFERENCES tags (id) NOT NULL
);

CREATE INDEX objectives_tags_objective_id_idx ON objectives_tags USING BTREE (objective_id);
CREATE INDEX objectives_tags_tag_id_idx ON objectives_tags USING BTREE (tag_id);
