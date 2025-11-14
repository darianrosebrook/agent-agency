#!/usr/bin/env python3
"""
Fix missing relevance fields in Evidence struct initializations
"""

import re
import sys

def fix_evidence_relevance(file_path):
    """Fix missing relevance fields in Evidence struct initializations"""
    
    with open(file_path, 'r') as f:
        content = f.read()
    
    # Pattern to match Evidence struct initialization missing relevance field
    # Look for confidence field followed by timestamp without relevance
    pattern = r'(confidence: [^,]+,\s*timestamp: Utc::now\(\),)'
    
    def replacement(match):
        confidence_line = match.group(1)
        # Extract the confidence value to determine relevance
        confidence_match = re.search(r'confidence: ([^,]+)', confidence_line)
        if confidence_match:
            confidence_val = confidence_match.group(1).strip()
            # Use confidence as relevance, or default to 0.8
            if confidence_val == 'complexity_score':
                relevance = '0.8'  # Default relevance for complexity
            elif confidence_val == 'confidence':
                relevance = '0.9'  # High relevance for documentation
            else:
                relevance = '0.8'  # Default relevance
        else:
            relevance = '0.8'  # Default relevance
            
        return f'confidence: {confidence_val},\n                relevance: {relevance}, // Default relevance score\n                timestamp: Utc::now(),'
    
    # Apply the fix
    fixed_content = re.sub(pattern, replacement, content)
    
    # Write back to file
    with open(file_path, 'w') as f:
        f.write(fixed_content)
    
    print(f"Fixed Evidence relevance fields in {file_path}")

if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("Usage: python fix_evidence_relevance.py <file_path>")
        sys.exit(1)
    
    fix_evidence_relevance(sys.argv[1])


