DROP TRIGGER tags_ts_name_update on tags;

ALTER TABLE tags
DROP COLUMN ts_config_name,
DROP COLUMN ts_name;