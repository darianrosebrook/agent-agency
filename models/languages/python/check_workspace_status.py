#!/usr/bin/env python3
"""
Check compilation status of workspace crates
"""

import subprocess
import json
from pathlib import Path

def get_workspace_members():
    """Get all workspace members from Cargo.toml"""
    try:
        with open("Cargo.toml", "r") as f:
            content = f.read()
            
        # Extract members
        members = []
        in_members = False
        for line in content.split('\n'):
            line = line.strip()
            if line.startswith('members = ['):
                in_members = True
                continue
            elif in_members and line.startswith(']'):
                break
            elif in_members and line.strip().startswith('"') and line.strip().endswith('",'):
                member = line.strip().strip('",')
                if not member.startswith('#'):  # Skip commented out members
                    members.append(member)
                    
        return members
    except:
        return []

def check_crate(crate_name):
    """Check if a crate compiles"""
    try:
        result = subprocess.run(['cargo', 'check', '-p', crate_name, '--lib'], 
                              capture_output=True, text=True, timeout=60)
        
        error_count = result.stderr.count('error[')
        warning_count = result.stderr.count('warning:')
        
        return {
            'name': crate_name,
            'success': result.returncode == 0,
            'errors': error_count,
            'warnings': warning_count,
            'exit_code': result.returncode
        }
    except subprocess.TimeoutExpired:
        return {
            'name': crate_name,
            'success': False,
            'errors': -1,  # Timeout
            'warnings': 0,
            'exit_code': -1
        }
    except Exception as e:
        return {
            'name': crate_name,
            'success': False,
            'errors': -2,  # Other error
            'warnings': 0,
            'exit_code': -2
        }

def main():
    print("🔍 WORKSPACE CRATE COMPILATION STATUS")
    print("=" * 50)
    
    members = get_workspace_members()
    
    if not members:
        print("❌ Could not parse workspace members")
        return
    
    print(f"📦 Found {len(members)} workspace members\n")
    
    results = []
    for member in members:
        print(f"🔧 Checking {member}...")
        result = check_crate(member)
        results.append(result)
        
        status = "✅" if result['success'] else "❌"
        if result['errors'] == -1:
            error_str = "TIMEOUT"
        elif result['errors'] == -2:
            error_str = "ERROR"
        else:
            error_str = f"{result['errors']} errors"
            
        print(f"   {status} {error_str}, {result['warnings']} warnings")
    
    print("\n📊 SUMMARY:")
    print("-" * 30)
    
    successful = [r for r in results if r['success']]
    failed = [r for r in results if not r['success']]
    
    print(f"✅ Successfully compiled: {len(successful)} crates")
    print(f"❌ Failed to compile: {len(failed)} crates")
    
    if successful:
        print("\n🎉 WORKING CRATES:")
        for result in successful:
            print(f"   ✅ {result['name']} ({result['warnings']} warnings)")
    
    if failed:
        print("\n💥 BROKEN CRATES:")
        for result in failed:
            print(f"   ❌ {result['name']} ({result['errors']} errors)")
    
    # Calculate overall health
    total_errors = sum(r['errors'] for r in results if r['errors'] > 0)
    print(f"\n🏥 OVERALL HEALTH: {len(successful)}/{len(members)} crates working ({total_errors} total errors)")

if __name__ == "__main__":
    main()
