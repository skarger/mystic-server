CREATE TABLE objectives (
    id SERIAL PRIMARY KEY,
    description TEXT NOT NULL,
    ts_config_name REGCONFIG NOT NULL DEFAULT 'english',
    ts_description TSVECTOR NOT NULL
);

CREATE INDEX objectives_ts_config_name_idx ON objectives USING BTREE (ts_config_name);
CREATE INDEX objectives_ts_description_idx ON objectives USING GIN (ts_description);

CREATE TRIGGER objectives_ts_description_update
BEFORE INSERT OR UPDATE ON objectives
FOR EACH ROW EXECUTE PROCEDURE
tsvector_update_trigger_column(ts_description, ts_config_name, description);

