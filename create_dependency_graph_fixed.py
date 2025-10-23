#!/usr/bin/env python3
"""
Create a detailed dependency graph showing torch/tch and apple-silicon relationships
"""

import os
import json
from pathlib import Path
import subprocess

def get_detailed_deps():
    """Get detailed dependency information"""
    
    # Check workspace Cargo.toml for torch dependencies
    workspace_toml = Path("iterations/v3/Cargo.toml")
    torch_workspace_deps = []
    
    if workspace_toml.exists():
        with open(workspace_toml, 'r') as f:
            content = f.read()
            if 'tch =' in content:
                torch_workspace_deps.append('tch')
            if 'torch-sys =' in content:
                torch_workspace_deps.append('torch-sys')
    
    # Check individual crate dependencies
    crates = [
        'apple-silicon', 'council', 'runtime-optimization', 'orchestration',
        'model-benchmarking', 'model-hotswap', 'self-prompting-agent', 'integration-tests'
    ]
    
    dependency_map = {}
    
    for crate in crates:
        cargo_path = Path(f"iterations/v3/{crate}/Cargo.toml")
        if cargo_path.exists():
            deps = []
            try:
                with open(cargo_path, 'r') as f:
                    content = f.read()
                    
                # Look for apple-silicon dependency
                if 'agent-agency-apple-silicon' in content or 'apple-silicon' in content:
                    deps.append('apple-silicon')
                
                # Look for torch dependencies
                if 'tch' in content:
                    deps.append('tch')
                if 'torch-sys' in content:
                    deps.append('torch-sys')
                    
                # Check for torch feature usage
                lib_rs = Path(f"iterations/v3/{crate}/src/lib.rs")
                if lib_rs.exists():
                    with open(lib_rs, 'r') as f:
                        lib_content = f.read()
                        if 'torch' in lib_content.lower():
                            deps.append('torch-feature')
                
            except:
                pass
            
            dependency_map[crate] = deps
    
    return torch_workspace_deps, dependency_map

def create_mermaid_graph(torch_workspace_deps, dependency_map):
    """Create a Mermaid graph"""
    
    graph = ["graph TD"]
    graph.append("    %% Torch/PyTorch Dependencies")
    graph.append("    Torch[PyTorch/torch] --> Tch[tch crate]")
    graph.append("    Tch --> TorchSys[torch-sys]")
    
    graph.append("    %% Apple Silicon Crate")
    graph.append("    AppleSilicon[apple-silicon crate<br/>🍎 Core ML, ANE, Metal]")
    graph.append("    AppleSilicon --> Tch")
    
    graph.append("    %% Dependent Crates")
    
    # Add dependency relationships
    for crate, deps in dependency_map.items():
        crate_name = crate.replace('-', '_')
        
        # Add crate node with features
        features = []
        if 'apple-silicon' in deps:
            features.append("🍎")
        if 'tch' in deps or 'torch-sys' in deps:
            features.append("🔥")
        if 'torch-feature' in deps:
            features.append("🧠")
        
        feature_str = " ".join(features) if features else ""
        graph.append(f"    {crate_name}[{crate}<br/>{feature_str}]")
        
        # Add edges
        if 'apple-silicon' in deps:
            graph.append(f"    {crate_name} --> AppleSilicon")
        if 'tch' in deps:
            graph.append(f"    {crate_name} --> Tch")
        if 'torch-sys' in deps:
            graph.append(f"    {crate_name} --> TorchSys")
    
    graph.append("    %% Styling")
    graph.append("    classDef torchClass fill:#ff6b6b,color:#fff,stroke:#d63031")
    graph.append("    classDef siliconClass fill:#0984e3,color:#fff,stroke:#0984e3")
    graph.append("    classDef dependentClass fill:#00b894,color:#fff,stroke:#00b894")
    graph.append("    class Tch,TorchSys,Torch torchClass")
    graph.append("    class AppleSilicon siliconClass")
    graph.append("    class council,runtime_optimization,orchestration,model_benchmarking,model_hotswap,self_prompting_agent,integration_tests dependentClass")
    
    return "\n".join(graph)

def create_text_report(torch_workspace_deps, dependency_map):
    """Create a detailed text report"""
    
    report = []
    report.append("🔍 AGENT AGENCY V3 TORCH/APPLE SILICON DEPENDENCY ANALYSIS")
    report.append("=" * 70)
    
    report.append("\n📋 WORKSPACE-LEVEL TORCH DEPENDENCIES:")
    report.append("-" * 40)
    for dep in torch_workspace_deps:
        report.append(f"  📦 {dep}")
    
    report.append("\n🍎 APPLE SILICON DEPENDENCY CHAIN:")
    report.append("-" * 40)
    
    silicon_users = [crate for crate, deps in dependency_map.items() if 'apple-silicon' in deps]
    for crate in silicon_users:
        deps = dependency_map[crate]
        torch_indicator = "🔥" if any(d in ['tch', 'torch-sys', 'torch-feature'] for d in deps) else ""
        report.append(f"  └─ {crate} {torch_indicator}")
    
    report.append("\n🔥 DIRECT TORCH USERS:")
    report.append("-" * 25)
    
    torch_users = [crate for crate, deps in dependency_map.items() if any(d in ['tch', 'torch-sys'] for d in deps)]
    for crate in torch_users:
        deps = dependency_map[crate]
        torch_deps = [d for d in deps if d in ['tch', 'torch-sys']]
        report.append(f"  ┌─ {crate}")
        for dep in torch_deps:
            report.append(f"  │  └─ uses {dep}")
    
    report.append("\n⚠️  CRITICAL RISK ANALYSIS:")
    report.append("-" * 30)
    
    if not silicon_users:
        report.append("  🚨 HIGH RISK: No crates depend on apple-silicon!")
        report.append("     Apple Silicon optimizations may be lost!")
    else:
        report.append(f"  ✅ {len(silicon_users)} crates depend on apple-silicon")
    
    if not torch_users and not torch_workspace_deps:
        report.append("  🚨 CRITICAL: No torch dependencies found!")
        report.append("     PyTorch integration may be completely broken!")
    else:
        total_torch = len(torch_users) + len(torch_workspace_deps)
        report.append(f"  ✅ {total_torch} torch dependencies found")
    
    # Check for indirect dependencies
    indirect_deps = []
    for crate, deps in dependency_map.items():
        if 'apple-silicon' in deps and any(d in ['tch', 'torch-sys', 'torch-feature'] for d in deps):
            indirect_deps.append(crate)
    
    if indirect_deps:
        report.append("\n🎯 TORCH + APPLE SILICON INTEGRATION:")
        report.append("-" * 40)
        for crate in indirect_deps:
            report.append(f"  🔗 {crate} (uses both torch and apple-silicon)")
    
    report.append("\n💡 PROTECTION STRATEGIES:")
    report.append("-" * 25)
    report.append("  1. Never disable apple-silicon crate in workspace")
    report.append("  2. Keep torch workspace dependencies intact")
    report.append("  3. Monitor dependency chains during refactoring")
    report.append("  4. Test torch features before major changes")
    report.append(f"  5. {len(silicon_users)} crates would break if apple-silicon is disabled")
    
    return "\n".join(report)

if __name__ == "__main__":
    torch_workspace_deps, dependency_map = get_detailed_deps()
    
    # Create Mermaid graph
    mermaid_graph = create_mermaid_graph(torch_workspace_deps, dependency_map)
    
    # Create text report
    text_report = create_text_report(torch_workspace_deps, dependency_map)
    
    # Write outputs
    with open("torch_dependency_graph.md", "w") as f:
        f.write("# Torch/Apple Silicon Dependency Graph\n\n")
        f.write("## Mermaid Diagram\n\n")
        f.write("```mermaid\n")
        f.write(mermaid_graph)
        f.write("\n```\n\n")
        f.write("## Detailed Analysis\n\n")
        f.write(text_report)
    
    print("📊 Dependency analysis complete!")
    print("📄 Report saved to: torch_dependency_graph.md")
    print()
    print(text_report)
