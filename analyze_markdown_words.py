#!/usr/bin/env python3
"""Analyze word patterns in markdown filenames."""

import sys
import re
from collections import Counter

# Prohibited patterns from our rules
PROHIBITED_PATTERNS = {
    'SUMMARY': ['summary', 'summaries'],
    'STATUS': ['status'],
    'ROADMAP': ['roadmap'],
    'CHECKLIST': ['checklist'],
    'PROGRESS': ['progress'],
    'AUDIT': ['audit'],
    'REPORT': ['report', 'reports'],
    'ASSESSMENT': ['assessment', 'assessments'],
    'ANALYSIS': ['analysis', 'analyses'],
    'INVESTIGATION': ['investigation', 'investigations'],
    'EVALUATION': ['evaluation', 'evaluations'],
    'COMPLETE': ['complete', 'completion'],
    'FIX': ['fix', 'fixes'],
    'PLAN': ['plan', 'plans'],
    'IMPLEMENTATION': ['implementation', 'implementations'],
    'TODO': ['todo', 'todos'],
    'SESSION': ['session', 'sessions'],
}

# Read all filenames from stdin
files = [line.strip() for line in sys.stdin if line.strip()]

# Extract just the filenames (not paths)
filenames = [f.split('/')[-1] for f in files]

# Extract words from filenames
words = []
for filename in filenames:
    # Remove .md extension
    name = filename.replace('.md', '')
    # Split by underscores, hyphens, and camelCase boundaries
    name = name.replace('_', ' ').replace('-', ' ').replace('.', ' ')
    name = re.sub(r'([a-z])([A-Z])', r'\1 \2', name)
    parts = name.split()
    words.extend([p.lower() for p in parts if p])

# Count word frequency
word_counts = Counter(words)

# Build a mapping of prohibited words to patterns
prohibited_words = {}
for pattern, word_list in PROHIBITED_PATTERNS.items():
    for word in word_list:
        prohibited_words[word] = pattern

# Categorize words
prohibited_found = {}
other_words = {}

for word, count in word_counts.items():
    if word in prohibited_words:
        pattern = prohibited_words[word]
        if pattern not in prohibited_found:
            prohibited_found[pattern] = {}
        prohibited_found[pattern][word] = count
    else:
        other_words[word] = count

# Print results
print("=" * 80)
print("PROHIBITED PATTERN ANALYSIS")
print("=" * 80)
print(f"\nTotal files analyzed: {len(filenames)}")
print(f"Total unique words: {len(word_counts)}")
print(f"Prohibited patterns found: {len(prohibited_found)}")
print("\n" + "=" * 80)
print("PROHIBITED WORDS BY PATTERN:")
print("=" * 80)

for pattern in sorted(prohibited_found.keys()):
    print(f"\n[{pattern}] - Pattern: *_{pattern}.md")
    print("-" * 60)
    for word, count in sorted(prohibited_found[pattern].items(), key=lambda x: -x[1]):
        print(f"  {count:4d} occurrences: '{word}'")
    total = sum(prohibited_found[pattern].values())
    print(f"  {'---':>60}")
    print(f"  {total:4d} TOTAL for this pattern")

print("\n" + "=" * 80)
print("OTHER COMMON WORDS (not prohibited):")
print("=" * 80)
print("\nTop 30:")
print("-" * 60)
for word, count in sorted(other_words.items(), key=lambda x: -x[1])[:30]:
    print(f"  {count:4d}  {word}")

# Save detailed breakdown to file
with open('markdown_word_analysis.txt', 'w') as f:
    f.write("MARKDOWN FILENAME WORD ANALYSIS\n")
    f.write("=" * 80 + "\n\n")
    f.write(f"Total files: {len(filenames)}\n")
    f.write(f"Total unique words: {len(word_counts)}\n\n")

    f.write("PROHIBITED PATTERNS BREAKDOWN:\n")
    f.write("-" * 80 + "\n\n")
    for pattern in sorted(prohibited_found.keys()):
        f.write(f"[{pattern}]\n")
        for word, count in sorted(prohibited_found[pattern].items(), key=lambda x: -x[1]):
            f.write(f"  {count:4d}  {word}\n")
        total = sum(prohibited_found[pattern].values())
        f.write(f"  TOTAL: {total}\n\n")

    f.write("\nALL WORD FREQUENCIES (sorted by frequency):\n")
    f.write("-" * 80 + "\n\n")
    for word, count in word_counts.most_common():
        is_prohibited = " [PROHIBITED]" if word in prohibited_words else ""
        f.write(f"{count:4d}  {word}{is_prohibited}\n")

print(f"\n\nDetailed analysis saved to: markdown_word_analysis.txt")







