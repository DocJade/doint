-- back to ints!
-- Update the bank
ALTER TABLE bank
CHANGE COLUMN `doints_on_hand` `doints_on_hand` INT NOT NULL COMMENT 'How much money the bank currently has.' ,
CHANGE COLUMN `total_doints` `total_doints` INT NOT NULL COMMENT 'How many doints are within the entire economy' ;

-- Update the users
ALTER TABLE users
CHANGE COLUMN `bal` `bal` BIGINT UNSIGNED NOT NULL ;

-- Fee table
ALTER TABLE fees
CHANGE COLUMN `flat_fee` `flat_fee` INT NOT NULL COMMENT 'The flat rate of every action a player can take thats money related.' ;