# Torch/Apple Silicon Dependency Graph

## Mermaid Diagram

```mermaid
graph TD
    %% Torch/PyTorch Dependencies
    Torch[PyTorch/torch] --> Tch[tch crate]
    Tch --> TorchSys[torch-sys]
    %% Apple Silicon Crate
    AppleSilicon[apple-silicon crate<br/>🍎 Core ML, ANE, Metal]
    AppleSilicon --> Tch
    %% Dependent Crates
    apple_silicon[apple-silicon<br/>🍎]
    apple_silicon --> AppleSilicon
    council[council<br/>🍎]
    council --> AppleSilicon
    runtime_optimization[runtime-optimization<br/>🍎]
    runtime_optimization --> AppleSilicon
    orchestration[orchestration<br/>🍎]
    orchestration --> AppleSilicon
    model_benchmarking[model-benchmarking<br/>🍎]
    model_benchmarking --> AppleSilicon
    model_hotswap[model-hotswap<br/>🍎]
    model_hotswap --> AppleSilicon
    self_prompting_agent[self-prompting-agent<br/>🍎]
    self_prompting_agent --> AppleSilicon
    integration_tests[integration-tests<br/>🍎 🔥]
    integration_tests --> AppleSilicon
    integration_tests --> Tch
    %% Styling
    classDef torchClass fill:#ff6b6b,color:#fff,stroke:#d63031
    classDef siliconClass fill:#0984e3,color:#fff,stroke:#0984e3
    classDef dependentClass fill:#00b894,color:#fff,stroke:#00b894
    class Tch,TorchSys,Torch torchClass
    class AppleSilicon siliconClass
    class council,runtime_optimization,orchestration,model_benchmarking,model_hotswap,self_prompting_agent,integration_tests dependentClass
```

## Detailed Analysis

🔍 AGENT AGENCY V3 TORCH/APPLE SILICON DEPENDENCY ANALYSIS
======================================================================

📋 WORKSPACE-LEVEL TORCH DEPENDENCIES:
----------------------------------------
  📦 tch
  📦 torch-sys

🍎 APPLE SILICON DEPENDENCY CHAIN:
----------------------------------------
  └─ apple-silicon 
  └─ council 
  └─ runtime-optimization 
  └─ orchestration 
  └─ model-benchmarking 
  └─ model-hotswap 
  └─ self-prompting-agent 
  └─ integration-tests 🔥

🔥 DIRECT TORCH USERS:
-------------------------
  ┌─ integration-tests
  │  └─ uses tch

⚠️  CRITICAL RISK ANALYSIS:
------------------------------
  ✅ 8 crates depend on apple-silicon
  ✅ 3 torch dependencies found

🎯 TORCH + APPLE SILICON INTEGRATION:
----------------------------------------
  🔗 integration-tests (uses both torch and apple-silicon)

💡 PROTECTION STRATEGIES:
-------------------------
  1. Never disable apple-silicon crate in workspace
  2. Keep torch workspace dependencies intact
  3. Monitor dependency chains during refactoring
  4. Test torch features before major changes
  5. 8 crates would break if apple-silicon is disabled