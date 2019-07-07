UPDATE tags SET ts_name = to_tsvector(name);

ALTER TABLE tags ALTER COLUMN ts_config_name SET NOT NULL;
ALTER TABLE tags ALTER COLUMN ts_name SET NOT NULL;