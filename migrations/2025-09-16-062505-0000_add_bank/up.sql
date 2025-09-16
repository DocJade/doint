-- Create the bank
CREATE TABLE bank (
id CHAR(1) NOT NULL DEFAULT 'X' COMMENT 'Prevent there from being more than one database row.',
doints_on_hand INT NOT NULL COMMENT 'How much money the bank currently has.',
total_doints INT NOT NULL COMMENT 'How many doints are within the entire economy',
tax_rate SMALLINT NOT NULL CHECK (tax_rate >= 0 AND tax_rate <= 1000) COMMENT 'Tax rate expressed as xxx.x%',
CONSTRAINT PK_T1 PRIMARY KEY (id),
CONSTRAINT CK_T1_Locked CHECK (id='X'),
CONSTRAINT CK_Cant_Have_More_Than_In_Circulation CHECK (doints_on_hand <= total_doints)
);
-- Create the single row.
-- No default doints in the DB, admins must add.
-- Default tax rate of 0.0%
INSERT INTO bank (id, doints_on_hand, total_doints, tax_rate)
VALUES ('X', 0, 0, 0);