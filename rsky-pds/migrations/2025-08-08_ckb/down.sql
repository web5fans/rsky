-- This file should undo anything in `up.sql`
ALTER TABLE pds.actor
    DROP COLUMN IF EXISTS ckbAddress;