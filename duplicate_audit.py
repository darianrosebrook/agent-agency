#!/usr/bin/env python3
"""
Comprehensive Duplicate Detection Script for Agent Agency V3
Identifies true duplicates across all crates and provides detailed analysis
"""

import os
import hashlib
import json
from pathlib import Path
from collections import defaultdict
import difflib

def get_file_hash(filepath):
    """Calculate SHA256 hash of file content"""
    try:
        with open(filepath, 'rb') as f:
            return hashlib.sha256(f.read()).hexdigest()
    except:
        return None

def get_file_content(filepath):
    """Get file content for comparison"""
    try:
        with open(filepath, 'r', encoding='utf-8') as f:
            return f.read()
    except:
        return None

def normalize_content(content):
    """Normalize content for comparison (remove comments, whitespace)"""
    if not content:
        return ""
    
    lines = content.split('\n')
    normalized = []
    
    for line in lines:
        # Remove comments (// and /* */)
        line = line.split('//')[0] if '//' in line else line
        line = line.split('/*')[0] if '/*' in line else line
        
        # Remove extra whitespace
        line = line.strip()
        
        # Skip empty lines
        if line:
            normalized.append(line)
    
    return '\n'.join(normalized)

def find_rust_files(base_path):
    """Find all Rust source files"""
    rust_files = []
    for root, dirs, files in os.walk(base_path):
        # Skip target directories
        if 'target' in root or 'node_modules' in root:
            continue
            
        for file in files:
            if file.endswith('.rs'):
                rust_files.append(os.path.join(root, file))
    
    return rust_files

def analyze_crate_structure(base_path):
    """Analyze the structure of each crate"""
    crates = {}
    
    for item in os.listdir(base_path):
        item_path = os.path.join(base_path, item)
        if os.path.isdir(item_path) and os.path.exists(os.path.join(item_path, 'Cargo.toml')):
            crates[item] = {
                'path': item_path,
                'src_files': [],
                'modules': set()
            }
            
            # Find all source files
            src_dir = os.path.join(item_path, 'src')
            if os.path.exists(src_dir):
                for root, dirs, files in os.walk(src_dir):
                    for file in files:
                        if file.endswith('.rs'):
                            file_path = os.path.join(root, file)
                            crates[item]['src_files'].append(file_path)
                            
                            # Extract module name
                            rel_path = os.path.relpath(file_path, src_dir)
                            module_name = rel_path.replace(os.sep, '::').replace('.rs', '')
                            if module_name == 'lib':
                                module_name = 'root'
                            crates[item]['modules'].add(module_name)
    
    return crates

def find_exact_duplicates(files):
    """Find files with identical content"""
    hash_to_files = defaultdict(list)
    
    for filepath in files:
        file_hash = get_file_hash(filepath)
        if file_hash:
            hash_to_files[file_hash].append(filepath)
    
    # Return groups with more than one file
    return {h: files for h, files in hash_to_files.items() if len(files) > 1}

def find_similar_files(files, threshold=0.8):
    """Find files with similar content"""
    similar_groups = []
    processed = set()
    
    for i, file1 in enumerate(files):
        if file1 in processed:
            continue
            
        content1 = get_file_content(file1)
        if not content1:
            continue
            
        normalized1 = normalize_content(content1)
        if not normalized1:
            continue
            
        similar_group = [file1]
        processed.add(file1)
        
        for j, file2 in enumerate(files[i+1:], i+1):
            if file2 in processed:
                continue
                
            content2 = get_file_content(file2)
            if not content2:
                continue
                
            normalized2 = normalize_content(content2)
            if not normalized2:
                continue
            
            # Calculate similarity
            similarity = difflib.SequenceMatcher(None, normalized1, normalized2).ratio()
            
            if similarity >= threshold:
                similar_group.append(file2)
                processed.add(file2)
        
        if len(similar_group) > 1:
            similar_groups.append(similar_group)
    
    return similar_groups

def analyze_module_duplicates(crates):
    """Analyze duplicate modules across crates"""
    module_to_crates = defaultdict(list)
    
    for crate_name, crate_info in crates.items():
        for module in crate_info['modules']:
            module_to_crates[module].append(crate_name)
    
    # Find modules that exist in multiple crates
    duplicate_modules = {m: crates for m, crates in module_to_crates.items() if len(crates) > 1}
    
    return duplicate_modules

def generate_report(base_path):
    """Generate comprehensive duplicate analysis report"""
    print(f"🔍 AGENT AGENCY DUPLICATE AUDIT - {base_path}")
    print("=" * 60)
    
    # Analyze crate structure
    print("\n📦 Analyzing crate structure...")
    crates = analyze_crate_structure(base_path)
    
    print(f"Found {len(crates)} crates:")
    for crate_name, crate_info in crates.items():
        print(f"  📁 {crate_name}: {len(crate_info['src_files'])} files, {len(crate_info['modules'])} modules")
    
    # Find all Rust files
    print("\n🔍 Scanning for Rust files...")
    all_rust_files = find_rust_files(base_path)
    print(f"Found {len(all_rust_files)} Rust files total")
    
    # Find exact duplicates
    print("\n🎯 Finding exact duplicates...")
    exact_duplicates = find_exact_duplicates(all_rust_files)
    
    if exact_duplicates:
        print(f"Found {len(exact_duplicates)} groups of exact duplicates:")
        for i, (hash_val, files) in enumerate(exact_duplicates.items(), 1):
            print(f"\n  Group {i} ({len(files)} files):")
            for file_path in files:
                rel_path = os.path.relpath(file_path, base_path)
                print(f"    📄 {rel_path}")
    else:
        print("✅ No exact duplicates found")
    
    # Skip similar files analysis for now (too slow)
    print("\n🔍 Skipping similar files analysis (too slow for large codebases)")
    
    # Analyze module duplicates
    print("\n📋 Analyzing module duplicates...")
    duplicate_modules = analyze_module_duplicates(crates)
    
    if duplicate_modules:
        print(f"Found {len(duplicate_modules)} modules duplicated across crates:")
        for module, crate_list in duplicate_modules.items():
            print(f"\n  Module '{module}' exists in:")
            for crate in crate_list:
                print(f"    📦 {crate}")
    else:
        print("✅ No duplicate modules found")
    
    # Generate recommendations
    print("\n🎯 RECOMMENDATIONS:")
    print("-" * 20)
    
    if exact_duplicates:
        print("❌ CRITICAL: Exact duplicates found - these should be consolidated")
        print("   - Create shared modules in a common crate")
        print("   - Use workspace dependencies to share code")
        print("   - Remove duplicate files after consolidation")
    
    if duplicate_modules:
        print("⚠️  WARNING: Duplicate module names across crates")
        print("   - Consider namespace prefixes")
        print("   - Move shared modules to common crate")
    
    if not exact_duplicates and not duplicate_modules:
        print("✅ No duplicates found - codebase is well organized!")
    
    return {
        'crates': {name: {
            'path': info['path'],
            'file_count': len(info['src_files']),
            'modules': list(info['modules'])
        } for name, info in crates.items()},
        'exact_duplicates': {h: files for h, files in exact_duplicates.items()},
        'duplicate_modules': duplicate_modules
    }

if __name__ == "__main__":
    import sys
    
    # Check all iterations
    iterations = ["iterations/v2", "iterations/v3", "iterations/v4"]
    all_results = {}
    
    for iteration in iterations:
        if os.path.exists(iteration):
            print(f"\n{'='*80}")
            print(f"CHECKING {iteration.upper()}")
            print(f"{'='*80}")
            
            try:
                result = generate_report(iteration)
                all_results[iteration] = result
            except Exception as e:
                print(f"❌ Error analyzing {iteration}: {e}")
                continue
        else:
            print(f"⚠️  {iteration} not found, skipping...")
    
    # Summary across all iterations
    print(f"\n{'='*80}")
    print("SUMMARY ACROSS ALL ITERATIONS")
    print(f"{'='*80}")
    
    total_duplicates = 0
    for iteration, result in all_results.items():
        duplicates = len(result.get('exact_duplicates', {}))
        total_duplicates += duplicates
        print(f"{iteration}: {duplicates} duplicate groups")
    
    print(f"\nTOTAL DUPLICATES FOUND: {total_duplicates}")
    
    if total_duplicates == 0:
        print("✅ No duplicates found across any iteration!")
    else:
        print("❌ Duplicates found - see individual reports above for details")
