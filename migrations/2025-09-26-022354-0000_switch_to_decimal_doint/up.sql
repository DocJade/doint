-- Update the bank
ALTER TABLE bank
CHANGE COLUMN `doints_on_hand` `doints_on_hand` DECIMAL(16,2) NOT NULL COMMENT 'How much money the bank currently has.' ,
CHANGE COLUMN `total_doints` `total_doints` DECIMAL(16,2) NOT NULL COMMENT 'How many doints are within the entire economy' ;

-- Update the users
ALTER TABLE users
CHANGE COLUMN `bal` `bal` DECIMAL(16,2) NOT NULL ;

-- Fee table
ALTER TABLE fees
CHANGE COLUMN `flat_fee` `flat_fee` DECIMAL(16,2) NOT NULL COMMENT 'The flat rate of every action a player can take thats money related.' ;