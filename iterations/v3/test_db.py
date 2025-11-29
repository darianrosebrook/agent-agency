import socket
import sys

try:
    import psycopg2
    conn = psycopg2.connect(
        dbname="agent_agency",
        user="agent_agency",
        password="secure_password",
        host="127.0.0.1",
        port="5432"
    )
    print("Connection successful!")
    conn.close()
except ImportError:
    print("psycopg2 not installed")
except Exception as e:
    print(f"Connection failed: {e}")
