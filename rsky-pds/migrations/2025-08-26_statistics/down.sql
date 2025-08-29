-- This file should undo anything in `up.sql`
DROP TABLE pds.access_token;

ALTER TABLE pds.refresh_token
    DROP COLUMN IF EXISTS loginTimes;