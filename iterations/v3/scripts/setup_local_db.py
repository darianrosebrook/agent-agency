#!/usr/bin/env python3
"""Setup local PostgreSQL database for agent-agency

This script creates the necessary user and database, enables pgvector,
and can optionally run migrations.

Usage:
    python3 setup_local_db.py
"""

import sys
import subprocess
import os

def run_sql_command(sql, database="postgres", user=None):
    """Run a SQL command using psql"""
    cmd = ["/opt/homebrew/opt/postgresql@17/bin/psql"]
    
    if user:
        cmd.extend(["-U", user])
    
    cmd.extend(["-h", "127.0.0.1", "-p", "5432", "-d", database, "-c", sql])
    
    # Set environment to avoid password prompts
    env = os.environ.copy()
    env["PGPASSWORD"] = ""  # Empty password for trust auth
    
    try:
        result = subprocess.run(
            cmd,
            env=env,
            capture_output=True,
            text=True,
            check=False
        )
        return result.returncode == 0, result.stdout, result.stderr
    except Exception as e:
        return False, "", str(e)

def main():
    print("Setting up local PostgreSQL database...")
    
    # Step 1: Create agent_agency user
    print("\n1. Creating agent_agency user...")
    sql = """
    DO $$
    BEGIN
        IF NOT EXISTS (SELECT FROM pg_user WHERE usename = 'agent_agency') THEN
            CREATE USER agent_agency WITH PASSWORD 'agent_agency_dev' SUPERUSER;
            RAISE NOTICE 'User agent_agency created';
        ELSE
            RAISE NOTICE 'User agent_agency already exists';
        END IF;
    END $$;
    """
    
    success, stdout, stderr = run_sql_command(sql)
    if success:
        print("✓ User created or already exists")
    else:
        print(f"✗ Failed to create user: {stderr}")
        # Try alternative: alter existing user
        alt_sql = "ALTER USER agent_agency WITH PASSWORD 'agent_agency_dev' SUPERUSER;"
        success2, _, _ = run_sql_command(alt_sql)
        if success2:
            print("✓ User password updated")
        else:
            print("Warning: Could not create/update user. Continuing anyway...")
    
    # Step 2: Create database (should already exist from createdb)
    print("\n2. Verifying agent_agency database exists...")
    check_sql = "SELECT 1 FROM pg_database WHERE datname = 'agent_agency';"
    success, stdout, stderr = run_sql_command(check_sql)
    if success and "1" in stdout:
        print("✓ Database exists")
    else:
        print("✗ Database does not exist. Please create it with: createdb agent_agency")
        return 1
    
    # Step 3: Enable pgvector extension
    print("\n3. Enabling pgvector extension...")
    pgvector_sql = "CREATE EXTENSION IF NOT EXISTS vector;"
    success, stdout, stderr = run_sql_command(pgvector_sql, database="agent_agency", user="agent_agency")
    if success:
        print("✓ pgvector extension enabled")
    else:
        print(f"✗ Failed to enable pgvector: {stderr}")
        print("You may need to install pgvector: brew install pgvector")
        return 1
    
    print("\n✓ Database setup complete!")
    print("\nTo use this database, set:")
    print("  export DATABASE_URL=\"postgresql://agent_agency:agent_agency_dev@localhost:5432/agent_agency\"")
    
    return 0

if __name__ == "__main__":
    sys.exit(main())





