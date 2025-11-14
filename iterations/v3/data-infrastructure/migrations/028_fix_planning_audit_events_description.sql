-- Migration 028: Fix planning_audit_events description column
-- Adds the description column if it's missing from planning_audit_events table
-- This fixes the "column description does not exist" error during UnifiedOrchestrator initialization

-- Check if column exists, and add it if missing
DO $$
BEGIN
    -- Check if planning_audit_events table exists
    IF EXISTS (
        SELECT 1 
        FROM information_schema.tables 
        WHERE table_name = 'planning_audit_events'
    ) THEN
        -- Check if description column exists
        IF NOT EXISTS (
            SELECT 1 
            FROM information_schema.columns 
            WHERE table_name = 'planning_audit_events' 
            AND column_name = 'description'
        ) THEN
            -- Add description column
            ALTER TABLE planning_audit_events 
            ADD COLUMN description TEXT NOT NULL DEFAULT '';
            
            -- Update existing rows to have a default description if empty
            UPDATE planning_audit_events 
            SET description = COALESCE(
                metadata->>'description',
                event_type || ' event for plan ' || plan_id::text
            )
            WHERE description = '';
            
            RAISE NOTICE 'Added description column to planning_audit_events table';
        ELSE
            RAISE NOTICE 'Description column already exists in planning_audit_events table';
        END IF;
    ELSE
        RAISE NOTICE 'planning_audit_events table does not exist - migration 005 may not have run';
    END IF;
END $$;

-- Verify the column was added successfully
DO $$
BEGIN
    IF EXISTS (
        SELECT 1 
        FROM information_schema.columns 
        WHERE table_name = 'planning_audit_events' 
        AND column_name = 'description'
    ) THEN
        RAISE NOTICE 'Verification: description column exists in planning_audit_events table';
    ELSE
        RAISE WARNING 'Verification failed: description column not found in planning_audit_events table';
    END IF;
END $$;

