#!/usr/bin/env python3
"""
Recovery Analyzer for Agent Agency Project
Analyzes recovered_work directory and generates reconstruction plan
"""

import os
import json
import re
from pathlib import Path
from collections import defaultdict, Counter
from datetime import datetime
import argparse

class RecoveryAnalyzer:
    def __init__(self, recovered_work_dir):
        self.recovered_work_dir = Path(recovered_work_dir)
        self.entries = []
        self.file_mappings = defaultdict(list)
        self.component_analysis = defaultdict(list)
        self.timeline = []
        
    def analyze_all_entries(self):
        """Analyze all entries.json files in recovery directories"""
        print(f"🔍 Analyzing {self.recovered_work_dir}...")
        
        for recovery_dir in self.recovered_work_dir.iterdir():
            if recovery_dir.is_dir():
                entries_file = recovery_dir / "entries.json"
                if entries_file.exists():
                    try:
                        with open(entries_file, 'r') as f:
                            data = json.load(f)
                            
                        # Extract file path from resource
                        resource_path = data.get('resource', '')
                        if 'agent-agency' in resource_path:
                            # Extract the relative path within agent-agency
                            match = re.search(r'agent-agency/(.+)', resource_path)
                            if match:
                                relative_path = match.group(1)
                                
                                # Parse each entry
                                for entry in data.get('entries', []):
                                    entry_data = {
                                        'recovery_id': recovery_dir.name,
                                        'file_id': entry['id'],
                                        'source': entry.get('source', 'Unknown'),
                                        'timestamp': entry.get('timestamp', 0),
                                        'resource_path': resource_path,
                                        'relative_path': relative_path,
                                        'recovery_dir': str(recovery_dir)
                                    }
                                    
                                    self.entries.append(entry_data)
                                    self.file_mappings[relative_path].append(entry_data)
                                    
                                    # Categorize by component
                                    component = self._categorize_component(relative_path)
                                    self.component_analysis[component].append(entry_data)
                                    
                                    # Add to timeline
                                    if entry.get('timestamp'):
                                        dt = datetime.fromtimestamp(entry['timestamp'] / 1000)
                                        self.timeline.append((dt, entry_data))
                                    
                    except Exception as e:
                        print(f"⚠️  Error processing {entries_file}: {e}")
        
        # Sort timeline
        self.timeline.sort(key=lambda x: x[0])
        
        print(f"✅ Found {len(self.entries)} recovery entries")
        print(f"📁 Affecting {len(self.file_mappings)} unique files")
        print(f"🧩 Across {len(self.component_analysis)} components")
        
    def _categorize_component(self, file_path):
        """Categorize file by component based on path"""
        path_parts = file_path.split('/')
        
        if 'iterations/v3' in file_path:
            if 'orchestration' in file_path:
                return 'orchestration'
            elif 'self-prompting-agent' in file_path:
                return 'self-prompting-agent'
            elif 'embedding-service' in file_path:
                return 'embedding-service'
            elif 'apple-silicon' in file_path:
                return 'apple-silicon'
            elif 'workers' in file_path:
                return 'workers'
            elif 'council' in file_path:
                return 'council'
            elif 'reflexive-learning' in file_path:
                return 'reflexive-learning'
            else:
                return 'v3-other'
        elif 'docs' in file_path:
            return 'documentation'
        elif '.caws' in file_path:
            return 'caws-config'
        else:
            return 'other'
    
    def generate_inventory_report(self):
        """Generate comprehensive inventory report"""
        report = []
        report.append("# Agent Agency Recovery Inventory Report")
        report.append(f"Generated: {datetime.now().isoformat()}")
        report.append(f"Total Recovery Entries: {len(self.entries)}")
        report.append(f"Unique Files Affected: {len(self.file_mappings)}")
        report.append("")
        
        # Component breakdown
        report.append("## Component Breakdown")
        for component, entries in sorted(self.component_analysis.items()):
            report.append(f"- **{component}**: {len(entries)} entries")
        report.append("")
        
        # Timeline analysis
        report.append("## Timeline Analysis")
        if self.timeline:
            earliest = self.timeline[0][0]
            latest = self.timeline[-1][0]
            report.append(f"- **Earliest Change**: {earliest}")
            report.append(f"- **Latest Change**: {latest}")
            report.append(f"- **Time Span**: {latest - earliest}")
        report.append("")
        
        # File mapping details
        report.append("## File Recovery Mapping")
        for file_path, entries in sorted(self.file_mappings.items()):
            # Get the latest entry for this file
            latest_entry = max(entries, key=lambda x: x['timestamp'])
            report.append(f"### {file_path}")
            report.append(f"- **Recovery ID**: {latest_entry['recovery_id']}")
            report.append(f"- **File ID**: {latest_entry['file_id']}")
            report.append(f"- **Source**: {latest_entry['source']}")
            report.append(f"- **Timestamp**: {datetime.fromtimestamp(latest_entry['timestamp'] / 1000)}")
            report.append(f"- **Total Versions**: {len(entries)}")
            report.append("")
        
        return "\n".join(report)
    
    def generate_reconstruction_plan(self):
        """Generate step-by-step reconstruction plan"""
        plan = []
        plan.append("# Agent Agency Reconstruction Plan")
        plan.append("")
        plan.append("## Phase 1: Critical Components (Priority 1)")
        plan.append("")
        
        # Prioritize by component importance and file count
        component_priority = [
            ('orchestration', 'Core orchestration system'),
            ('self-prompting-agent', 'Autonomous agent system'),
            ('workers', 'Worker execution system'),
            ('council', 'Constitutional AI system'),
            ('embedding-service', 'Embedding and tokenization'),
            ('apple-silicon', 'Apple Silicon integration'),
            ('reflexive-learning', 'Learning system'),
            ('caws-config', 'CAWS configuration'),
            ('documentation', 'Documentation updates'),
            ('v3-other', 'Other V3 components')
        ]
        
        for component, description in component_priority:
            if component in self.component_analysis:
                entries = self.component_analysis[component]
                plan.append(f"### {component.upper()}: {description}")
                plan.append(f"- **Files to restore**: {len(set(e['relative_path'] for e in entries))}")
                plan.append(f"- **Total entries**: {len(entries)}")
                
                # List specific files
                unique_files = set(e['relative_path'] for e in entries)
                for file_path in sorted(unique_files):
                    file_entries = [e for e in entries if e['relative_path'] == file_path]
                    latest = max(file_entries, key=lambda x: x['timestamp'])
                    plan.append(f"  - `{file_path}` (ID: {latest['file_id']}, Recovery: {latest['recovery_id']})")
                plan.append("")
        
        plan.append("## Phase 2: Reconstruction Commands")
        plan.append("")
        plan.append("### Step 1: Create Directory Structure")
        plan.append("```bash")
        unique_dirs = set()
        for file_path in self.file_mappings.keys():
            dir_path = os.path.dirname(file_path)
            if dir_path:
                unique_dirs.add(dir_path)
        
        for dir_path in sorted(unique_dirs):
            plan.append(f"mkdir -p '{dir_path}'")
        plan.append("```")
        plan.append("")
        
        plan.append("### Step 2: Restore Files")
        plan.append("```bash")
        for file_path, entries in sorted(self.file_mappings.items()):
            latest_entry = max(entries, key=lambda x: x['timestamp'])
            recovery_file = f"recovered_work/{latest_entry['recovery_id']}/{latest_entry['file_id']}"
            plan.append(f"cp '{recovery_file}' '{file_path}'")
        plan.append("```")
        plan.append("")
        
        plan.append("## Phase 3: Validation Checklist")
        plan.append("")
        plan.append("- [ ] All directories created")
        plan.append("- [ ] All files restored")
        plan.append("- [ ] Compilation errors fixed")
        plan.append("- [ ] Dependencies updated")
        plan.append("- [ ] Tests passing")
        plan.append("- [ ] Integration validated")
        
        return "\n".join(plan)
    
    def generate_recovery_script(self):
        """Generate automated recovery script"""
        script = []
        script.append("#!/bin/bash")
        script.append("# Automated Recovery Script for Agent Agency")
        script.append("set -e")
        script.append("")
        script.append("echo '🚀 Starting Agent Agency Recovery...'")
        script.append("")
        
        # Create directories
        script.append("echo '📁 Creating directory structure...'")
        unique_dirs = set()
        for file_path in self.file_mappings.keys():
            dir_path = os.path.dirname(file_path)
            if dir_path:
                unique_dirs.add(dir_path)
        
        for dir_path in sorted(unique_dirs):
            script.append(f"mkdir -p '{dir_path}'")
        script.append("")
        
        # Restore files
        script.append("echo '📄 Restoring files...'")
        for file_path, entries in sorted(self.file_mappings.items()):
            latest_entry = max(entries, key=lambda x: x['timestamp'])
            recovery_file = f"recovered_work/{latest_entry['recovery_id']}/{latest_entry['file_id']}"
            script.append(f"echo 'Restoring {file_path}...'")
            script.append(f"cp '{recovery_file}' '{file_path}'")
        script.append("")
        
        script.append("echo '✅ Recovery complete!'")
        script.append("echo '🔧 Next: Fix compilation errors and test integration'")
        
        return "\n".join(script)

def main():
    parser = argparse.ArgumentParser(description='Analyze recovered work and generate reconstruction plan')
    parser.add_argument('--recovered-dir', default='recovered_work', help='Path to recovered_work directory')
    parser.add_argument('--output-dir', default='recovery_output', help='Output directory for reports')
    args = parser.parse_args()
    
    # Create output directory
    output_dir = Path(args.output_dir)
    output_dir.mkdir(exist_ok=True)
    
    # Initialize analyzer
    analyzer = RecoveryAnalyzer(args.recovered_dir)
    
    # Analyze all entries
    analyzer.analyze_all_entries()
    
    # Generate reports
    print("📊 Generating reports...")
    
    # Inventory report
    inventory = analyzer.generate_inventory_report()
    with open(output_dir / 'inventory_report.md', 'w') as f:
        f.write(inventory)
    print(f"📋 Inventory report: {output_dir / 'inventory_report.md'}")
    
    # Reconstruction plan
    plan = analyzer.generate_reconstruction_plan()
    with open(output_dir / 'reconstruction_plan.md', 'w') as f:
        f.write(plan)
    print(f"📋 Reconstruction plan: {output_dir / 'reconstruction_plan.md'}")
    
    # Recovery script
    script = analyzer.generate_recovery_script()
    with open(output_dir / 'recover.sh', 'w') as f:
        f.write(script)
    os.chmod(output_dir / 'recover.sh', 0o755)
    print(f"🔧 Recovery script: {output_dir / 'recover.sh'}")
    
    # Summary
    print("\n" + "="*60)
    print("📊 RECOVERY ANALYSIS SUMMARY")
    print("="*60)
    print(f"Total recovery entries: {len(analyzer.entries)}")
    print(f"Unique files affected: {len(analyzer.file_mappings)}")
    print(f"Components identified: {len(analyzer.component_analysis)}")
    
    print("\n🧩 COMPONENT BREAKDOWN:")
    for component, entries in sorted(analyzer.component_analysis.items()):
        unique_files = len(set(e['relative_path'] for e in entries))
        print(f"  {component}: {unique_files} files, {len(entries)} entries")
    
    print(f"\n📁 Output files created in: {output_dir}")
    print("🚀 Ready to reconstruct!")

if __name__ == "__main__":
    main()
