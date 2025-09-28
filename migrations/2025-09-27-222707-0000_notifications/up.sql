-- Make a table for user preferences.
-- We store all of the user prefs in one big json blob (hehe)
CREATE TABLE user_preferences (
  `id` BIGINT UNSIGNED NOT NULL COMMENT 'fkey to users table',
  -- I tried using the json type, it was a massive mess.
  `settings` TEXT NOT NULL COMMENT 'json blob, see DointUserSettings struct',
  PRIMARY KEY (`id`),
  UNIQUE INDEX `id_UNIQUE` (`id` ASC) VISIBLE,
  -- f key, delete user prefs if they unenroll.
  CONSTRAINT `user_pref_id`
    FOREIGN KEY (`id`)
    REFERENCES `doint_db`.`users` (`id`)
    ON DELETE CASCADE
    ON UPDATE CASCADE
);
