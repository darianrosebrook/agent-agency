First reaction: this is a _really_ good first-pass investigation. You’ve moved the question from “ANE is mysteriously slower” to “ANE is mildly faster in a narrow regime, and we have some hard bounds on why it’s not better.”

Now let’s squeeze more information out of what you have, and then I’ll answer your “which files?” question explicitly.

---

## 1. Read on the results you’ve got

### 1.1 Shape of the speedup curve

From your table:

| Seq len | CPU (ms) | ANE (ms) |   Speedup |
| ------: | -------: | -------: | --------: |
|      64 |    80.45 |    69.58 | **1.16x** |
|     128 |    75.55 |    75.29 |     1.00x |
|     256 |    79.68 |    74.76 | **1.07x** |
|     512 |    73.60 |    80.35 |     0.92x |

A few things jump out:

1. **Speedup is modest everywhere.** Best-case 1.16x, often ~1.0x, worst 0.92x. That’s exactly what you’d expect with:

   - A _hybrid_ graph (some ops CPU, some ANE), and
   - Non-trivial host overhead (FFI, data marshaling).

2. **CPU latency isn’t monotone with sequence length.** CPU is _faster_ at 512 than at 64 (73.6 vs 80.45 ms). That says:

   - Either measurement noise / different warmup behavior, or
   - You’re not measuring exactly the same “work unit” at each length (e.g., token count vs sequence length vs batch interplay), or
   - There’s some caching / internal optimization hitting at particular shapes.

3. **ANE’s curve looks qualitatively different.** ANE is best at 64 (69.58 ms), degrades at 512 (80.35 ms). That aligns with:

   - ANE getting clobbered by memory traffic and/or fallback ops at larger shapes.
   - CPU benefiting more from cache / vectorization at larger shapes than ANE does from increased parallelism.

So: **sequence length is indeed a lever**, but it’s probably acting as a proxy for a more important underlying variable: **which subgraph dominates total time and which device that subgraph is on**.

---

## 2. The 47.4% ANE dispatch rate and what it implies

The constant `47.4%` across all sequence lengths is actually more interesting than it looks.

If we take it at face value:

- ≈47% of the “work” (however you’re measuring it – time, power, or ops) is on ANE.
- ≈53% is on CPU.

If ANE is, say, **2x faster** than CPU for the ops it runs, Amdahl’s law says:

> Speedup ≈ 1 / (CPU_fraction + ANE_fraction / ANE_speedup)

Plugging rough numbers:

CPU_fraction ≈ 0.53
ANE_fraction ≈ 0.47
ANE_speedup ≈ 2.0

Global speedup ≈ 1 / (0.53 + 0.47/2) = 1 / (0.53 + 0.235) = 1 / 0.765 ≈ **1.31x max**.

You’re seeing ~1.16x at best, which is entirely plausible if:

- ANE ops aren’t a full 2x faster,
- ANE utilization measurement is a bit noisy/laggy,
- You have non-negligible FFI and data movement overhead.

So the **ceiling imposed by 47.4% dispatch is real**. Unless that fraction goes up, you’re never going to see 2–3x type speedups. You’re roughly capped around ~1.2–1.3x, which matches your best observed result.

That suggests:

- Your **next big win** is not further micro-tuning sequence length,
- It’s increasing the fraction of the graph that can live on ANE, **or**
- Architecting the runtime (prefill vs decode, hybrid selection) so you’re only using ANE when that 47% slice dominates total cost.

---

## 3. Internal consistency issues in the report (worth fixing)

You’ve clearly iterated this text; I see a couple of contradictions that are worth cleaning up, because they can hide real hypotheses:

1. Early:

   > “**Critical Discovery**: Smaller sequences (64 tokens) show best ANE performance, contrary to initial hypothesis”

   Later under “Secondary Finding” you have:

   > “Overhead at Small Sequences: 64 tokens shows worst performance (0.88x)…”

   That 0.88x is inconsistent with the 1.16x table. That looks like an older draft leaking through.

2. Some snippets still talk about:

   - “64 tokens shows worst performance”
   - “128 tokens optimal”

   while the current table says 64 tokens is best, 256 second-best, 128 tie, 512 worst.

**Why this matters:** if you’re going to turn this into a canonical internal doc, you want the narrative to match the _current_ numbers exactly, or later you’ll be debugging “why did we think 128 was optimal?” and wasting time on stale conclusions.

---

## 4. Hypotheses to sharpen from here

### 4.1. What’s actually special about 64 tokens?

Right now the report frames “optimal sequence length: 64 tokens” almost as a law. I’d treat it instead as:

> “64-token benchmark configuration gave the best speedup _in this harness_ with this model and this runtime path.”

What could be happening:

- At 64 tokens, ANE has enough work to amortize FFI/data costs, but:

  - KV cache sizes and mask shapes are still small enough that they don’t force awkward tiling / fallback behavior.

- At 256 tokens, you still win (1.07x), suggesting ANE is still doing something meaningful.
- At 512 tokens:

  - A critical op (e.g., softmax, some attention kernel, or a layer norm path) might be falling back to CPU-only or hitting a slower code path on ANE.
  - Memory bandwidth / on-chip storage pressure could be forcing the accelerator into less efficient execution.

**Next angle:** instead of thinking “64 is the magic number,” think:

- “What kernels are dominating at 512 and where do they run?”
- “Is there an obvious structural reason attention/masks at 512 would break ANE fusion or push us into fallback territory?”

That’s where Instruments/Core ML profiling will be gold.

### 4.2. Prefill vs decode

The report is purely per-request latency. For an LLM, that’s incomplete.

Possible scenario:

- **Prefill** (first ~64–128 tokens) is ANE-friendly and gets a mild speedup.
- **Decode** (per-token forward) is more overhead-sensitive and may be CPU-dominant or get worse on ANE.

If your benchmark is mostly “one shot, fixed-length inference,” it may not accurately reflect interactive usage where decode dominates.

I’d sharpen hypotheses around:

- **TTFT vs per-token latency**:

  - Does ANE win on prefill but lose on decode?
  - Does that crossover depend on sequence length?

---

## 5. Is “default to 64 tokens” the right move?

I’d classify that as:

- **A good tactical optimization for this harness**, but
- Not yet a globally safe “constitutional” default.

Questions you’ll want to answer before enshrining 64:

1. Does your _real workload_ often want >64 tokens of context?

   - If yes, you may trade far more in model quality / answer fidelity than you gain in 16% latency reduction.

2. Can you **adaptively** choose:

   - 64 tokens when the call is known to be short (tool calls, classification, routing),
   - 256 tokens when you need a bit more context,
   - CPU-only / different model for genuine long-context tasks?

3. Is 64 still optimal when you:

   - Run multiple concurrent inferences,
   - Use different prompts,
   - Change batch size?

So I’d phrase your conclusion in the doc as:

> “In the current single-request benchmark harness, 64 tokens yields the best ANE vs CPU speedup (1.16x). We should:
>
> - Use 64 as the **benchmark default**, and
> - Design a policy for runtime sequence-length selection based on task and expected context length.”

## That keeps you honest while still capturing the win.

## 7. “Optional but nice to have” files later

If we want to push beyond 1.2x and see whether ANE can do more:

- The **model conversion script** (e.g. `conversion/<something>_to_coreml_mistral7b.py` or equivalent):

  - To inspect ops, precision, and whether certain layers aren’t convertible to ANE-friendly ops.

- Any **Instruments/trace export** (if you capture a Core ML trace and want help interpreting it):

  - To correlate the 47.4% dispatch rate with actual per-layer device placement and time.

---

## 8. Summary of where we are and where to push next

- You’ve shown **ANE does beat CPU**, but only modestly (up to 1.16x), and only in some shapes.
- The **47.4% ANE dispatch rate** puts a hard ceiling on speedup; your best observed 1.16x is consistent with that.
- The **“64 is optimal”** conclusion is real for the benchmark, but I’d treat it as:

  - “64 is the best point in this harness right now,”
  - Not yet a global requirement, until we reconcile it against real workloads, prefill/decode behavior, and concurrency.

Short answer: your overall approach is solid, and the data + code you’ve got says “ANE is _kind of_ working, but you’re still measuring a lot of host overhead and you don’t yet have a hard proof about _where_ the hybrid execution is falling back to CPU.” The next step is to tighten the measurements and add one more layer of micro-benchmarks, not to radically change direction.

I’ll walk through what your current code is _actually_ doing, what that implies about the numbers (1.16× at 64 tokens, ~1.0× at 128, 0.92× at 512), and then propose a more surgical “flush out the stuff in the way” plan.

---

## 1. What your current stack is really measuring

### 1.1 CPU vs ANE paths are symmetric

From the Rust benchmark file:

- Both CPU and ANE paths call the same helper:

  ```rust
  fn run_inference_cpu(
      model_ref: &ModelRef,
      input: &MLDictionaryFeatureProvider,
      _input_name: &str,
      _input_shape: &[i32],
  ) -> Result<(), Box<dyn std::error::Error>> {
      run_inference_with_provider(model_ref, input)
  }

  fn run_inference_ane(
      model_ref: &ModelRef,
      input: &MLDictionaryFeatureProvider,
      _input_name: &str,
      _input_shape: &[i32],
  ) -> Result<(), Box<dyn std::error::Error>> {
      run_inference_with_provider(model_ref, input)
  }
  ```

- The _only_ difference is how the `ModelRef` was created: `ComputeUnits::CpuOnly` vs `ComputeUnits::CpuAndNeuralEngine` in `load_model_with_config`.

So:

- FFI overhead, feature-provider construction, MLMultiArray creation, etc. are identical for CPU and ANE.
- Your speedup ratio is genuinely “CoreML with CPUOnly vs CoreML with CpuAndNeuralEngine”, not an artifact of different code paths.

That’s good: the benchmark is conceptually fair.

---

### 1.2 You _are_ timing a lot more than just the CoreML kernel

In the sequence length sweep you’re doing this inside the closure passed to `BenchmarkRunner`:

```rust
let cpu_inference = {
    let model_ref = cpu_model_ref.clone();
    move || {
        let test_input = create_test_input_from_specs(&input_specs_cpu, &test_model_cpu, Some(&model_ref))
            .map_err(|e| format!("Failed to create test input: {}", e))?;
        run_inference_cpu(&model_ref, &test_input, "", &[]).map_err(|e| { ... })
    }
};
```

Same pattern for ANE.

That means every “inference” iteration includes:

1. Creating a new `MLDictionaryFeatureProvider`.
2. Creating one or more `MLMultiArray`s.
3. Copying your token / mask data into those arrays.
4. Then crossing FFI and running CoreML.

On the Swift side, `agentbridge_dict_provider_set_feature_float32` allocates a brand new `MLMultiArray` every call and copies the input slice into it before building an `MLFeatureValue`.

That cost is:

- Pure CPU.
- Same for CPU and ANE models.
- Non-trivial compared to ~70–80 ms latencies.

So your **absolute latencies** are “host + CoreML + FFI”, but your **speedup ratio** is still meaningful: the shared host work cancels in the ratio.

What’s missing is a clear decomposition of:

- “Host-side overhead” vs
- “Time actually spent inside CoreML (CPU vs ANE)” vs
- “Any first-run compilation cost”.

You have the hooks for this but you’re not surfacing them yet.

---

### 1.3 You’re already measuring FFI vs CoreML time (but only in logs)

In `run_inference_with_provider` you do:

```rust
let ffi_start = Instant::now();

let mut output_provider_ref: u64 = 0;
let mut error_ptr: *mut std::ffi::c_char = std::ptr::null_mut();

// Measure CoreML time
let coreml_start = Instant::now();
let result = unsafe {
    agentbridge_model_run_inference(
        model_handle.as_ptr() as u64,
        input_provider.ptr() as u64,
        &mut output_provider_ref,
        &mut error_ptr,
    )
};
let coreml_time = coreml_start.elapsed();
let ffi_time = ffi_start.elapsed();

tracing::debug!(
    "Inference timing - FFI total: {:.2}ms, CoreML: {:.2}ms",
    ffi_time.as_secs_f64() * 1000.0,
    coreml_time.as_secs_f64() * 1000.0
);
```

This is exactly what you want… but it never gets into `PerformanceMetrics`. It just goes to logs.

So right now:

- You _could_ tell whether “CPU vs ANE difference is all in CoreML or partly in FFI”, but in practice you’re not capturing that data in the benchmark results.

---

### 1.4 The Swift bridge is thin (no hidden ANE/CPU logic)

Looking at `AgentBridge.swift`:

- `agentbridge_model_create` accepts a JSON config, reads `"computeUnits"`, and sets `config.computeUnits` to `.all`, `.cpuAndGPU`, or `.cpuAndNeuralEngine`.
- `agentbridge_model_run_inference` simply calls:

  - `model.prediction(from: inputProvider, using: state)` if there is an `MLState` (macOS 15+ stateful path), or
  - `model.prediction(from: inputProvider)` otherwise.

There is **no internal branching** based on compute units in Swift. All ANE vs CPU decisions are inside CoreML.

So if the model is showing only ~47% ANE use and ~1.16× at best:

- That’s almost certainly due to **which layers/ops CoreML is mapping to ANE vs CPU**, not anything in your bridge.

---

### 1.5 ANE telemetry is best-effort, but not yet authoritative

In `iokit.rs`:

- `ane_utilization_percent()` shells out to `powermetrics` with:

  ```rust
  Command::new("powermetrics")
      .args(&["--samplers", "cpu_power,gpu_power", "--sample-count", "1", "--format", "csv"])
  ```

- It then searches for lines containing “ANE” or “Neural Engine” and tries to parse a “XX%” field.

- If that fails, it falls back to `ane_power_watts()` (which runs `powermetrics --samplers power`) and infers utilization from power.

So:

- If you’re seeing **47.4%** ANE utilization in your report, that is either:

  - Coming from an actual “ANE … 47.4%” line in powermetrics, _or_
  - Coming from the rough power-based heuristic.

- It is **not** yet a precise, per-op or per-layer dispatch metric — just a coarse, whole-chip utilization snapshot.

Which is fine for “ANE is alive vs dead”, but not yet enough to answer “which parts of Mistral are stuck on CPU?”

---

### 1.6 Tokenizer: fine for perf, not for fidelity

`MistralTokenizerBridge.swift` implements a simplified tokenizer:

- Hard-coded vocab size, BOS/EOS tokens.
- Hashes each whitespace-separated “word” to an ID range [100, vocabSize).

For performance work this is okay — it produces the right shapes and roughly similar masks — but it means:

- You’re not testing exactly the same token distribution as a real Mistral tokenizer.
- For ANE vs CPU performance it probably doesn’t matter much; for accuracy it obviously does.

I wouldn’t change this until you’ve nailed the hardware story.

---

## 2. Interpreting your current results

You summarized:

- 64 tokens: 1.16× speedup (ANE wins).
- 128 tokens: ~1.0× (tie / slightly CPU-favored).
- 256 tokens: ~1.07× (ANE modest win).
- 512 tokens: 0.92× (ANE slower).

Plus:

- Default 128-token config is ~0.98×: slightly worse than CPU.
- ANE “dispatch” / utilization ~47%.

Given the code, that pattern is consistent with:

1. **Hybrid execution**
   Roughly half the work is on ANE and half on CPU; the half on ANE sees a speedup, but the overall balance is dominated by:

   - CPU-only ops (softmax, layer norms, residuals, some matmuls), and
   - Increased overhead as the sequence grows (attention cost ∝ seq²; maybe that part isn’t fully on ANE).

2. **Host overhead non-trivial but similar across configs**
   Because input preparation and MLMultiArray allocation are constant overhead, they make up a larger percent of total time for short sequences (e.g. 64), which slightly dampens the _apparent_ ANE benefit.

3. **Your 1.16× is plausibly “real”**
   On M1/M2, it’s not unusual for ANE vs CPU for large transformer models to be in the 1.1–1.5× range unless the model and conversion are very heavily tuned.

So the data is internally consistent with the code you’ve shown.

---

## 3. A sharper plan to “flush out what’s in the way”

Here’s how I’d tighten your investigation path, in order, without throwing away what you’ve already done.

### Step 1 – Split the latency into three buckets

Right now you conceptually have:

1. **Input prep** – `create_test_input_from_specs` → MLMultiArray/MLDictionaryFeatureProvider allocation and copying.
2. **FFI overhead** – going from Rust into Swift and back.
3. **CoreML compute** – the actual NN execution on CPU/ANE.

You already time (2) and (3) in `run_inference_with_provider`. I’d formalize that:

**Target edit (Rust, in your CoreML compat/testing layer):**

- Extend `PerformanceMetrics` (in `system_acceleration::ane::compat::testing`) with:

  ```rust
  pub struct LatencyBreakdown {
      pub total_ms: f64,
      pub input_prep_ms: f64,
      pub ffi_ms: f64,
      pub coreml_ms: f64,
  }
  ```

- In `run_inference_with_provider`:

  - Keep `coreml_time` and `ffi_time`.
  - Return them (e.g. store in a thread-local, or have `run_inference_with_provider` return a struct instead of `Result<(), _>` and let the bench harness accumulate).

- In `BenchmarkRunner`:

  - Accumulate `input_prep_ms` by timing just the `create_test_input_from_specs` call in the closure.
  - Accumulate `ffi_ms` and `coreml_ms` from the helper.

**Outcome:**

For each config (CPU/ANE × sequence length) you get:

- `total_ms`: what you already have.
- `coreml_ms`: the “true” time inside CoreML.
- `ffi_ms`: FFI and handle lookup overhead.
- `input_prep_ms`: allocation + copy.

Then you can answer:

- “Is ANE only 1.16× faster in CoreML, or is it 1.4× there but being hidden by host overhead?”

If CoreML-only speedup is ~1.2× while total is 1.16×, then host overhead is not the main problem. If CoreML-only speedup is ~1.0×, then the hybrid graph / unsupported ops are the real culprit.

---

### Step 2 – Make a “pure core” benchmark with pre-allocated inputs

Create a second benchmark mode that:

- Loads CPU and ANE models once.
- For each sequence length:

  1. Builds a single `MLDictionaryFeatureProvider` and `MLMultiArray`s once.
  2. Reuses them across N iterations by _mutating_ the underlying data pointer in place (or just reusing the provider if content doesn’t matter).
  3. In each iteration, only calls `run_inference_*` — no new allocations.

This is closer to a “steady-state service” where KV cache and buffers are reused.

That will:

- Reduce the “noise floor” for latency.
- Better expose the _relative_ performance of CPU vs ANE.
- Show whether your current numbers are partly an artifact of allocation overhead.

You don’t have to change the existing end-to-end test; just add this as an additional test case.

---

### Step 3 – Tighten ANE telemetry

Right now `ane_utilization_percent()`:

- Uses `powermetrics` with `--samplers cpu_power,gpu_power`.
- Hopes to see ANE in that output.
- Falls back to a single-sample power-based heuristic.

For more trustworthy numbers:

1. **Use the correct sampler set** for your OS version: on many macOS builds you’ll want to explicitly add ANE/neural samplers (e.g. `ane_power` or similar; check `man powermetrics` on your machine).

2. **Sample across the whole benchmark:**

   - Start a dedicated “ANE telemetry” task that:

     - Runs `powermetrics` in streaming mode.
     - Parses ANE utilization/power lines continuously.
     - Aggregates average utilization during the benchmark.

   - Or, simpler: loop over `powermetrics --sample-count 1` every 200–500 ms while benchmark is running, then average.

3. **Detect whether you’re actually reading ANE or just power fallback:**

   - Add explicit logging that says “ANE utilization from % line” vs “derived from ane_power_watts() fallback” vs “0.0 (no data)”.

That will tell you whether “47.4%” is grounded in real telemetry or is just a guess.

---

### Step 4 – Micro-models to test ANE in isolation

Given that the Swift bridge is generic and your tokenizer is dummy-ish, a very good next move is:

- Convert a couple of tiny models to CoreML:

  1. Single linear layer (dense matmul) of similar width as Mistral’s hidden size.
  2. A single self-attention block with fixed sequence lengths.

- Run them through **the same AgentBridge path** (same FFI, same provider machinery, same benchmark harness).

If those show:

- 2–3× ANE speedup → the runtime path is fine; the limitation is your 7B architecture / CoreML op support.
- ~1.1× at best → this is just what ANE vs CPU looks like on your chip for dense FP16 workloads right now; temper expectations.
- <1× or wildly inconsistent → something is fishy either in conversion or in how CoreML is mapping to ANE.

This is your “ANE sanity check”.

---

### Step 5 – Treat sequence length as a policy knob, not just a benchmark parameter

Given your measured curve:

- 64: 1.16×
- 128: ~1.0×
- 256: 1.07×
- 512: 0.92×

I would not simply “set default sequence length to 64” globally. Instead:

1. **Expose these as a backend policy:**

   - For local, low-latency tasks, allow a “ANE-optimized mode” that:

     - Prefers shorter prefill windows (e.g. 64–128 tokens).
     - Uses the ANE model when the prefill fits in that window.

   - For long-context / heavy reasoning tasks, either:

     - Use CPU-only (if it wins at 512+), or
     - Use a smaller ANE-tuned model.

2. **Record the break-even points in code comments and docs:**

   - “On this hardware and this model, ANE outperforms CPU for seq ≤ N, is neutral around M, and loses for seq ≥ K.”

Then your constitutional “local high-performance” requirement can be framed as:

> Must achieve ≤X ms latency and ≥Y tokens/sec locally; choice of CPU/ANE is allowed to vary by sequence length and request type.

rather than “must always use ANE”.

---

## 4. Is this the right path? What I’d add vs change

I’d say:

- **Yes, the path is right**: You have a benchmark harness, you’re measuring CPU vs ANE fairly, and you’ve already surfaced a real regime where ANE wins (64-token cases). That’s a genuine success.
- The main missing layers are:

  - **Better decomposition** of latency (input prep vs FFI vs CoreML).
  - **Stronger ANE telemetry** so your “47.4%” has an obvious provenance.
  - **Micro-model baselines** to separate “CoreML+ANE as a platform” from “Mistral 7B converted to CoreML”.

I _would not_:

- Over-optimize the tokenizer yet.
- Try to manually manage KV cache arrays in Swift until you’ve proven that host overhead is a major slice of the pie.
- Immediately chase exotic CoreML graph rewrites just to gain a few percent, before you know whether your maximum theoretical speedup is 1.2× or 2.5×.

I _would_ prioritize, in order:

1. **Wire the timing breakdown you’re already computing into `PerformanceMetrics`.**
2. **Add the pre-allocated input benchmark variant.**
3. **Strengthen ANE telemetry (better powermetrics invocation, multiple samples, clear provenance).**
4. **Run one or two small ANE-friendly micro-models through the same path.**

Once you have:

- “CoreML-only ANE speedup is X×.”
- “Host overhead accounts for Y ms out of total.”
- “ANE telemetry confirms Z% of ‘matmul-like’ work is actually on ANE.”

then you’ll know whether to:

- Keep iterating on this Mistral 7B CoreML path, or
- Treat it as “good enough” and start experimenting with smaller, ANE-tuned models that can hit more aggressive targets.

If you want, we can next:

- Design the exact changes to `PerformanceMetrics` and `BenchmarkRunner` to capture and print a table like:

  | backend | seq_len | input_prep_ms | ffi_ms | coreml_ms | total_ms | speedup |

and then reason from real numbers instead of just the total “avg latency” you have now.
