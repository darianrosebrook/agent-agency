# Intentionally broken Python file for arbiter testing
# This file contains multiple errors that the arbiter should fix

import json
import requests
from typing import Dict, List, Optional
from datetime import datetime

# Missing import
result = fetch_user_data(user_id)

# Type annotation issues
def calculate_total(items: List[int]) -> str:  # Should return int, not str
    return sum(items)

# Missing error handling
def risky_operation():
    data = json.loads("invalid json")  # This will raise JSONDecodeError
    return data

# Inconsistent naming convention
user_name = "john"  # Should be user_name (snake_case is correct in Python)
userAge = 25  # Should be user_age

# Unused variable
unused_var = "this should be removed or prefixed with underscore"

# Missing type annotations
config = {
    "api_url": "https://api.example.com",
    "timeout": 5000,
    "retries": 3
}

# TODO comment that should be addressed
# TODO: Implement proper error handling for API calls

# PLACEHOLDER: This is a placeholder that needs implementation
def placeholder_function():
    # PLACEHOLDER: Add actual implementation
    pass

# MOCK DATA: This should be replaced with real data
mock_users = [
    {"id": "1", "name": "John", "email": "john@example.com"},
    {"id": "2", "name": "Jane", "email": "jane@example.com"}
]

# Missing docstring
class User:
    def __init__(self, user_id: str, name: str, email: str):
        self.id = user_id
        self.name = name
        self.email = email
        self.created_at = datetime.now()

    def to_dict(self):
        return {
            "id": self.id,
            "name": self.name,
            "email": self.email,
            "created_at": self.created_at.isoformat()
        }

# Missing error handling in class method
def get_user_by_id(user_id: str) -> Optional[User]:
    # This should have proper error handling
    response = requests.get(f"https://api.example.com/users/{user_id}")
    data = response.json()  # This can fail
    return User(data["id"], data["name"], data["email"])

# Missing type hints
def process_users(users):
    results = []
    for user in users:
        if user["email"].endswith("@example.com"):
            results.append(user)
    return results

# Indentation error (intentional)
def broken_indentation():
print("This has wrong indentation")

# Missing return statement
def function_without_return(x: int) -> int:
    x * 2  # Should be: return x * 2

# Unreachable code
def unreachable_code():
    return "first return"
    return "second return"  # This will never be reached

if __name__ == "__main__":
    print("Hello, broken Python world!")

