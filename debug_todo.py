#!/usr/bin/env python3
"""Debug script to check a few TODO items manually."""

import json

def main():
    # The file is actually valid JSON - it's just minified
    with open("v3-todo-domain-aware-to-check-off.json", 'r') as f:
        todos = json.load(f)

    print(f"Loaded {len(todos)} items")

    # Check first few items
    for i, item in enumerate(todos[:5]):
        print(f"Item {i+1}:")
        print(f"  Keys: {list(item.keys())}")
        for key, value in item.items():
            print(f"  {key}: {repr(value)}")
        print()

if __name__ == "__main__":
    main()
