delete from objectives_tags;
delete from objectives_goal_areas;
delete from goal_areas;
delete from tags;
delete from objectives;

ALTER SEQUENCE goal_areas_id_seq RESTART;
ALTER SEQUENCE tags_id_seq RESTART;
ALTER SEQUENCE objectives_id_seq RESTART;
ALTER SEQUENCE objectives_goal_areas_id_seq RESTART;
ALTER SEQUENCE objectives_tags_id_seq RESTART;