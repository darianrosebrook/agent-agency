# engine-mps

Metal/MPS-backed inference engine for judge evaluation, leveraging `system-acceleration`'s Metal executor while the CoreML/ANE toolchain is disabled. Runs deterministic, real tensor compute on Apple Silicon GPUs to produce structured `JudgeEngine` responses.

## Status
- macOS + Metal only
- No CoreML/ANE; switch back once toolchain conflicts are resolved

## Tests
```bash
cargo test -p engine-mps --all-features
```




