"""Sample file for mutation testing."""

def add_numbers(a, b):
    """Add two numbers."""
    return a + b

def check_condition(x):
    """Check if x is greater than 10."""
    if x > 10:
        return True
    else:
        return False

def get_status(value):
    """Get status based on value."""
    if value == "active":
        return "running"
    elif value == "inactive":
        return "stopped"
    else:
        return "unknown"

# Some test code
result = add_numbers(5, 3)
assert result == 8

status = get_status("active")
assert status == "running"
