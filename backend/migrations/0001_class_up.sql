ALTER TABLE users ADD COLUMN graduation_year INT NOT NULL default 0;
ALTER TABLE users ADD COLUMN class_letter CHAR(1) NOT NULL default '';

ALTER TABLE users ALTER COLUMN graduation_year DROP default;
ALTER TABLE users ALTER COLUMN class_letter DROP default;