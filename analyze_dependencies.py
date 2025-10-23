#!/usr/bin/env python3
"""
Dependency Analysis Script for Agent Agency V3
Shows the dependency graph and identifies critical torch/tch dependencies
"""

import os
import json
from pathlib import Path
import subprocess
import sys

def find_cargo_tomls(base_path):
    """Find all Cargo.toml files in the project"""
    return list(Path(base_path).rglob("Cargo.toml"))

def parse_cargo_toml(filepath):
    """Parse a Cargo.toml file to extract dependencies"""
    try:
        result = subprocess.run(['cargo', 'toml', 'get', '--package-path', str(filepath)], 
                              capture_output=True, text=True, cwd=os.path.dirname(filepath))
        if result.returncode == 0:
            return json.loads(result.stdout)
    except:
        pass
    
    # Fallback: basic TOML parsing
    dependencies = {}
    try:
        with open(filepath, 'r') as f:
            content = f.read()
            
        # Extract dependencies section
        in_deps = False
        for line in content.split('\n'):
            line = line.strip()
            if line.startswith('[dependencies]'):
                in_deps = True
                continue
            elif line.startswith('[') and in_deps:
                break
            elif in_deps and '=' in line and not line.startswith('#'):
                parts = line.split('=', 1)
                if len(parts) == 2:
                    dep_name = parts[0].strip()
                    dep_value = parts[1].strip()
                    dependencies[dep_name] = dep_value
    except:
        pass
    
    return dependencies

def analyze_dependencies():
    """Analyze the dependency graph"""
    base_path = Path("iterations/v3")
    if not base_path.exists():
        print("Error: iterations/v3 directory not found")
        return
    
    # Find all crates
    crates = {}
    torch_users = set()
    apple_silicon_users = set()
    
    for cargo_file in find_cargo_tomls(base_path):
        crate_name = cargo_file.parent.name
        deps = parse_cargo_toml(cargo_file)
        
        crates[crate_name] = {
            'path': str(cargo_file.parent),
            'dependencies': deps,
            'uses_torch': False,
            'uses_apple_silicon': False
        }
        
        # Check for torch/tch usage
        for dep_name, dep_spec in deps.items():
            if 'tch' in dep_name.lower() or 'torch' in dep_name.lower():
                torch_users.add(crate_name)
                crates[crate_name]['uses_torch'] = True
            if 'apple-silicon' in dep_name.lower() or 'agent-agency-apple-silicon' in dep_name:
                apple_silicon_users.add(crate_name)
                crates[crate_name]['uses_apple_silicon'] = True
        
        # Check source code for torch/tch imports
        src_dir = cargo_file.parent / "src"
        if src_dir.exists():
            for rs_file in src_dir.rglob("*.rs"):
                try:
                    with open(rs_file, 'r') as f:
                        content = f.read()
                        if 'tch::' in content or 'torch::' in content:
                            torch_users.add(crate_name)
                            crates[crate_name]['uses_torch'] = True
                except:
                    pass
    
    return crates, torch_users, apple_silicon_users

def print_dependency_graph(crates, torch_users, apple_silicon_users):
    """Print a formatted dependency graph"""
    print("🔗 AGENT AGENCY V3 DEPENDENCY ANALYSIS")
    print("=" * 50)
    
    print("\n📦 CRATES USING TORCH/TCH:")
    print("-" * 30)
    for crate in sorted(torch_users):
        crate_info = crates.get(crate, {})
        path = crate_info.get('path', 'Unknown')
        print(f"  🔴 {crate} ({path})")
    
    print("\n�� CRATES USING APPLE SILICON:")
    print("-" * 30)
    for crate in sorted(apple_silicon_users):
        crate_info = crates.get(crate, {})
        path = crate_info.get('path', 'Unknown')
        print(f"  🍏 {crate} ({path})")
    
    print("\n🔍 CRITICAL DEPENDENCY CHAINS:")
    print("-" * 30)
    
    # Find crates that depend on torch users
    dependents = {}
    for crate_name, crate_info in crates.items():
        deps = crate_info.get('dependencies', {})
        for dep_name in deps.keys():
            if dep_name in torch_users:
                if dep_name not in dependents:
                    dependents[dep_name] = []
                dependents[dep_name].append(crate_name)
    
    for torch_crate, deps in dependents.items():
        print(f"  📋 {torch_crate} is used by:")
        for dep in sorted(deps):
            print(f"    └─ {dep}")
    
    print("\n⚠️  CRITICAL WARNINGS:")
    print("-" * 20)
    if not torch_users:
        print("  ❌ No crates found using torch/tch - this may indicate missing dependencies!")
    else:
        print(f"  ✅ Found {len(torch_users)} crates using torch/tch")
    
    if not apple_silicon_users:
        print("  ❌ No crates found using apple-silicon - dependency may be broken!")
    else:
        print(f"  ✅ Found {len(apple_silicon_users)} crates using apple-silicon")
    
    print("\n🎯 RECOMMENDATIONS:")
    print("-" * 20)
    if torch_users:
        print("  ✅ Torch functionality appears to be properly integrated")
        print("  ✅ Apple Silicon optimizations are being used")
    else:
        print("  ❌ Check torch/tch integration - may need workspace dependencies")
        print("  ❌ Verify apple-silicon crate is properly linked")

if __name__ == "__main__":
    crates, torch_users, apple_silicon_users = analyze_dependencies()
    print_dependency_graph(crates, torch_users, apple_silicon_users)
