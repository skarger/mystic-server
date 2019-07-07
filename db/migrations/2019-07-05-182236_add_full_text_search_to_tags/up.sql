ALTER TABLE tags
ADD COLUMN ts_config_name REGCONFIG DEFAULT 'english',
ADD COLUMN ts_name TSVECTOR;

CREATE INDEX tags_ts_config_name_idx ON tags USING BTREE (ts_config_name);
CREATE INDEX tags_ts_name_idx ON tags USING GIN (ts_name);

CREATE TRIGGER tags_ts_name_update
BEFORE INSERT OR UPDATE ON tags
FOR EACH ROW EXECUTE PROCEDURE
tsvector_update_trigger_column(ts_name, ts_config_name, name);