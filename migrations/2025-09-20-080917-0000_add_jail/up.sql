-- Your SQL goes here
CREATE TABLE jail (
  `id` BIGINT UNSIGNED NOT NULL COMMENT 'fkey to users table',
  `until` TIMESTAMP NOT NULL COMMENT 'When the user should be let out of jail. Everything is UTC based.',
  `reason` TINYTEXT NOT NULL COMMENT 'See the JailReason enum',
  `cause` TINYTEXT NOT NULL COMMENT 'See the JailCause enum',
  `can_bail` TINYINT(1) NOT NULL COMMENT 'Can this person be bailed out? 0/1',
  PRIMARY KEY (`id`),
  UNIQUE INDEX `id_UNIQUE` (`id` ASC) VISIBLE,
  -- f key, delete user from jail if user is deleted from dointdb
  CONSTRAINT `id`
    FOREIGN KEY (`id`)
    REFERENCES `doint_db`.`users` (`id`)
    ON DELETE CASCADE
    ON UPDATE CASCADE
);
