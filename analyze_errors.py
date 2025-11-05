import subprocess
import re
from collections import defaultdict

# Run cargo check and capture output
result = subprocess.run(['cargo', 'check', '--workspace'], 
                       capture_output=True, text=True, cwd='/Users/darianrosebrook/Desktop/Projects/agent-agency')

output = result.stdout + result.stderr

# Parse the output
crates = defaultdict(lambda: {'warnings': 0, 'errors': 0, 'details': []})
current_crate = None

lines = output.split('\n')
for line in lines:
    # Check for crate checking line
    crate_match = re.search(r'Checking (\w+) v\d+\.\d+\.\d+', line)
    if crate_match:
        current_crate = crate_match.group(1)
        continue
    
    # Check for error
    if 'error:' in line and current_crate:
        crates[current_crate]['errors'] += 1
        crates[current_crate]['details'].append(('error', line.strip()))
    
    # Check for warning
    if 'warning:' in line and current_crate:
        crates[current_crate]['warnings'] += 1
        crates[current_crate]['details'].append(('warning', line.strip()))

# Also check summary lines
for line in lines:
    summary_match = re.search(r'`(\w+)`.*generated (\d+) warnings', line)
    if summary_match:
        crate_name = summary_match.group(1)
        warning_count = int(summary_match.group(2))
        crates[crate_name]['warnings'] = warning_count
    
    error_match = re.search(r'`(\w+)`.*due to (\d+) previous error', line)
    if error_match:
        crate_name = error_match.group(1)
        error_count = int(error_match.group(2))
        crates[crate_name]['errors'] = error_count

# Print results
print("=== RUST CRATE ERROR ANALYSIS ===\n")

total_errors = 0
total_warnings = 0

for crate, data in sorted(crates.items()):
    if data['errors'] > 0 or data['warnings'] > 0:
        print(f"{crate}:")
        print(f"  Errors: {data['errors']}")
        print(f"  Warnings: {data['warnings']}")
        print(f"  Total Issues: {data['errors'] + data['warnings']}")
        print()
        
        total_errors += data['errors']
        total_warnings += data['warnings']

print(f"TOTAL ACROSS ALL CRATES:")
print(f"  Errors: {total_errors}")
print(f"  Warnings: {total_warnings}")
print(f"  Total Issues: {total_errors + total_warnings}")
