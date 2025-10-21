#!/usr/bin/env python3
"""Simple test file for mutation testing."""

def test_addition():
    """Test basic addition."""
    result = 2 + 3
    assert result == 5, f"Expected 5, got {result}"

def test_comparison():
    """Test comparison."""
    x = 15
    if x > 10:
        result = True
    else:
        result = False
    assert result == True, f"Expected True, got {result}"

def test_equality():
    """Test equality."""
    value = "active"
    if value == "active":
        status = "running"
    elif value == "inactive":
        status = "stopped"
    else:
        status = "unknown"
    assert status == "running", f"Expected 'running', got '{status}'"

if __name__ == "__main__":
    test_addition()
    test_comparison()
    test_equality()
    print("All tests passed!")
