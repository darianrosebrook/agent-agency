#!/usr/bin/env python3
"""
Categorize all TODO items from todo.md file into structured categories.
"""

import re
from collections import defaultdict
from pathlib import Path
from typing import Dict, List, Tuple, Set

# Category definitions with patterns
CATEGORIES = {
    "SCRUB_OUTDATED": {
        "patterns": [
            r"iterations/v2/",
            r"playground/",
            r"deprecated",
            r"old_",
        ],
        "description": "Outdated/playground code that can be removed",
        "action": "DELETE"
    },
    "KEEP_TESTS": {
        "patterns": [
            r".*test.*\.rs",
            r".*mock.*\.rs",
        ],
        "description": "Test/mock code - intentionally contains TODOs",
        "action": "KEEP"
    },
    "KEEP_FEATURE_FLAGS": {
        "patterns": [
            r"placeholder.*disabled",
            r"stub.*disabled",
            r"feature.*disabled",
            r"cfg\(feature",
        ],
        "description": "Intentional placeholders for disabled features",
        "action": "KEEP"
    },
    "KEEP_CRITICAL_V3": {
        "patterns": [
            r"iterations/v3/data-infrastructure",
            r"iterations/v3/agent-orchestration",
            r"iterations/v3/system-federated-ml",
        ],
        "description": "Critical V3 modules needing review",
        "action": "REVIEW"
    },
    "DEFER_ENHANCEMENTS": {
        "patterns": [
            r"sophisticated",
            r"comprehensive",
            r"advanced",
            r"future",
            r"enhance",
        ],
        "description": "Future enhancements - defer post-V3",
        "action": "DEFER"
    },
    "DEFER_DOCUMENTATION": {
        "patterns": [
            r"document",
            r"comment",
            r"doc",
        ],
        "description": "Documentation improvements - low priority",
        "action": "DEFER"
    },
}


def categorize_entry(file_path: str, line_num: int, content: str) -> Tuple[str, Dict]:
    """Categorize a single TODO entry."""
    content_lower = content.lower()
    
    # Check SCRUB_OUTDATED first
    if any(re.search(pattern, file_path, re.IGNORECASE) for pattern in CATEGORIES["SCRUB_OUTDATED"]["patterns"]):
        return "SCRUB_OUTDATED", {
            "priority": "LOW",
            "action": "DELETE",
            "reason": "Outdated/playground code"
        }
    
    # Check KEEP_TESTS
    if any(re.search(pattern, file_path, re.IGNORECASE) for pattern in CATEGORIES["KEEP_TESTS"]["patterns"]):
        return "KEEP_TESTS", {
            "priority": "LOW",
            "action": "KEEP",
            "reason": "Test/mock code"
        }
    
    # Check KEEP_FEATURE_FLAGS (content-based)
    if any(re.search(pattern, content_lower, re.IGNORECASE) for pattern in CATEGORIES["KEEP_FEATURE_FLAGS"]["patterns"]):
        return "KEEP_FEATURE_FLAGS", {
            "priority": "LOW",
            "action": "KEEP",
            "reason": "Intentional placeholder for disabled feature"
        }
    
    # Check KEEP_CRITICAL_V3
    if any(re.search(pattern, file_path, re.IGNORECASE) for pattern in CATEGORIES["KEEP_CRITICAL_V3"]["patterns"]):
        # Check if it's a real TODO or just test comment
        if any(word in content_lower for word in ["mock", "stub", "placeholder"]) and "test" in content_lower:
            return "KEEP_TESTS", {
                "priority": "LOW",
                "action": "KEEP",
                "reason": "Test code in critical module"
            }
        return "KEEP_CRITICAL_V3", {
            "priority": "HIGH",
            "action": "REVIEW",
            "reason": "Critical V3 module needs attention"
        }
    
    # Check DEFER_ENHANCEMENTS (content-based)
    if any(re.search(pattern, content_lower, re.IGNORECASE) for pattern in CATEGORIES["DEFER_ENHANCEMENTS"]["patterns"]):
        return "DEFER_ENHANCEMENTS", {
            "priority": "LOW",
            "action": "DEFER",
            "reason": "Future enhancement"
        }
    
    # Check DEFER_DOCUMENTATION (content-based)
    if any(re.search(pattern, content_lower, re.IGNORECASE) for pattern in CATEGORIES["DEFER_DOCUMENTATION"]["patterns"]):
        return "DEFER_DOCUMENTATION", {
            "priority": "LOW",
            "action": "DEFER",
            "reason": "Documentation improvement"
        }
    
    # Default: needs manual review
    return "REVIEW_NEEDED", {
        "priority": "MEDIUM",
        "action": "REVIEW",
        "reason": "Needs manual categorization"
    }


def parse_todo_file(file_path: Path) -> Dict:
    """Parse todo.md file and extract categorized entries."""
    categories = defaultdict(lambda: defaultdict(lambda: defaultdict(list)))
    
    current_file = None
    
    with open(file_path, 'r') as f:
        for line in f:
            line = line.rstrip()
            
            # Skip header lines
            if not line or line.startswith("results -") or line == "":
                continue
            
            # Check if this is a file header (ends with colon and contains a path)
            if line.endswith(':') and ('/' in line or '\\' in line):
                current_file = line[:-1]  # Remove trailing colon
                continue
            
            # If we have a current file and this looks like a line number entry
            if current_file and re.match(r'^\d+:', line):
                parts = line.split(':', 1)
                if len(parts) == 2:
                    try:
                        line_num = int(parts[0])
                        content = parts[1].strip()
                        
                        # Only process if it contains TODO-related keywords
                        if any(keyword in content.upper() for keyword in ['TODO', 'PLACEHOLDER', 'MOCK', 'STUB']):
                            category, metadata = categorize_entry(current_file, line_num, content)
                            categories[category][current_file][line_num] = {
                                "content": content,
                                "metadata": metadata
                            }
                    except ValueError:
                        # Not a valid line number, skip
                        pass
    
    return categories


def generate_output(categories: Dict, output_file: Path):
    """Generate categorized markdown output."""
    # Sort categories by priority order
    category_order = [
        ("KEEP_CRITICAL_V3", "🔴 Critical V3 Modules - Review Required"),
        ("REVIEW_NEEDED", "🟡 Needs Manual Review"),
        ("KEEP_FEATURE_FLAGS", "🟢 Feature Flags - Intentional"),
        ("KEEP_TESTS", "🟢 Test Code - Intentional"),
        ("DEFER_ENHANCEMENTS", "⏳ Future Enhancements - Defer"),
        ("DEFER_DOCUMENTATION", "⏳ Documentation - Defer"),
        ("SCRUB_OUTDATED", "🗑️ Outdated/Playground - Delete"),
    ]
    
    with open(output_file, 'w') as f:
        f.write("# TODO Items - Categorized\n\n")
        f.write("This file categorizes all TODO items from `todo.md` for easier review and cleanup.\n\n")
        f.write("---\n\n")
        
        total_files = 0
        total_entries = 0
        
        for category_key, category_title in category_order:
            if category_key not in categories or not categories[category_key]:
                continue
            
            cat_info = categories[category_key]
            file_count = len(cat_info)
            entry_count = sum(len(entries) for entries in cat_info.values())
            
            total_files += file_count
            total_entries += entry_count
            
            category_def = CATEGORIES.get(category_key, {})
            description = category_def.get("description", "No description")
            action = category_def.get("action", "UNKNOWN")
            
            f.write(f"## {category_title}\n\n")
            f.write(f"**Action**: {action}  \n")
            f.write(f"**Description**: {description}  \n")
            f.write(f"**Files**: {file_count}  \n")
            f.write(f"**Entries**: {entry_count}  \n\n")
            
            # Sort files alphabetically
            for file_path in sorted(cat_info.keys()):
                entries = cat_info[file_path]
                f.write(f"### {file_path}\n\n")
                f.write(f"**Total entries**: {len(entries)}  \n\n")
                
                # Show first few entries
                for line_num in sorted(entries.keys())[:5]:
                    entry = entries[line_num]
                    content = entry["content"]
                    metadata = entry["metadata"]
                    f.write(f"- Line {line_num}: `{content[:80]}{'...' if len(content) > 80 else ''}`  \n")
                    f.write(f"  - Priority: {metadata['priority']}  \n")
                
                if len(entries) > 5:
                    f.write(f"- ... ({len(entries) - 5} more entries)  \n")
                
                f.write("\n")
            
            f.write("---\n\n")
        
        # Summary
        f.write("## Summary\n\n")
        f.write(f"- **Total files categorized**: {total_files}\n")
        f.write(f"- **Total entries categorized**: {total_entries}\n\n")
        f.write("### Category Breakdown\n\n")
        for category_key, category_title in category_order:
            if category_key in categories and categories[category_key]:
                file_count = len(categories[category_key])
                entry_count = sum(len(entries) for entries in categories[category_key].values())
                f.write(f"- {category_title}: {file_count} files, {entry_count} entries\n")


def main():
    todo_file = Path(__file__).parent.parent / "todo.md"
    
    if not todo_file.exists():
        print(f"Error: {todo_file} not found")
        return 1
    
    print(f"Parsing {todo_file}...")
    categories = parse_todo_file(todo_file)
    
    # Generate categorized output
    output_file = Path(__file__).parent.parent / "todo_categorized.md"
    print(f"Generating categorized output to {output_file}...")
    generate_output(categories, output_file)
    
    print(f"\n✅ Categorized output written to: {output_file}")
    print(f"\n📊 Summary:")
    category_order = [
        "KEEP_CRITICAL_V3",
        "REVIEW_NEEDED",
        "KEEP_FEATURE_FLAGS",
        "KEEP_TESTS",
        "DEFER_ENHANCEMENTS",
        "DEFER_DOCUMENTATION",
        "SCRUB_OUTDATED",
    ]
    
    for category_key in category_order:
        if category_key in categories and categories[category_key]:
            file_count = len(categories[category_key])
            entry_count = sum(len(entries) for entries in categories[category_key].values())
            print(f"  {category_key}: {file_count} files, {entry_count} entries")
    
    return 0


if __name__ == "__main__":
    exit(main())
