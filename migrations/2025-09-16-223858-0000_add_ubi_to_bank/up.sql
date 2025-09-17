-- Your SQL goes here
ALTER TABLE bank
ADD COLUMN ubi_rate SMALLINT NOT NULL CHECK (ubi_rate >= 0 AND ubi_rate <= 1000) DEFAULT 0 COMMENT 'UBI rate expressed as xxx.x%';
