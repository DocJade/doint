-- Create fee tracker. Has a single entry as well for simplicity.
CREATE TABLE fees (
id CHAR(1) NOT NULL DEFAULT 'X' COMMENT 'Prevent there from being more than one database row.',
flat_fee INT NOT NULL CHECK (flat_fee >= 0) COMMENT 'The flat rate of every action a player can take thats money related.',
-- Expressed as xxx.x%
percentage_fee SMALLINT NOT NULL CHECK (percentage_fee >= 0 AND percentage_fee <= 1000) COMMENT 'The an additional fee on top of every transaction that is based on the amount of money used in the transaction. On top of the flat fee.',
CONSTRAINT PK_Fees PRIMARY KEY (id),
CONSTRAINT CK_Fees_Locked CHECK (id='X')
);
-- Create the single row.
-- No default doints in the DB, admins must add.
-- Flat fee rate of 50 doints per action.
-- Default percentage fee of 5&
INSERT INTO fees (id, flat_fee, percentage_fee)
VALUES ('X', 50, 50);