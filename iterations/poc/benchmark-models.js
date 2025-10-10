/**
 * Benchmark script to test inference speed of different Gemma models
 *
 * @author @darianrosebrook
 */

const { OllamaClient } = require("./dist/ai/index.js");

const MODELS_TO_TEST = [
  "gemma3n:e2b", // 5.6 GB - smaller but still capable
  "gemma3n:e4b", // 7.5 GB - largest gemma3n
  "gemma3:1b", // 815 MB - smallest gemma3
  "gemma3:4b", // 3.3 GB - medium gemma3
];

const TEST_PROMPT =
  "Transform this casual text into professional language: 'hey guys, this is pretty cool but we need to fix some stuff'";

async function benchmarkModel(modelName) {
  console.log(`\n🧪 Benchmarking ${modelName}...`);

  try {
    const client = new OllamaClient({
      model: modelName,
      host: "http://localhost:11434",
    });

    // Check if model is available
    const available = await client.isAvailable();
    if (!available) {
      console.log(`❌ ${modelName} is not available`);
      return null;
    }

    console.log(`✅ ${modelName} is available`);

    // Warm up the model
    console.log(`🔥 Warming up ${modelName}...`);
    await client.generate({
      prompt: "Hello",
      maxTokens: 10,
    });

    // Run benchmark
    console.log(`⚡ Running benchmark...`);
    const startTime = Date.now();

    const response = await client.generate({
      prompt: TEST_PROMPT,
      systemPrompt:
        "You are a professional editor. Transform casual text into formal, professional language.",
      config: {
        maxTokens: 100,
      },
    });

    const endTime = Date.now();
    const duration = endTime - startTime;
    const tokensGenerated = response.text.split(" ").length; // rough estimate
    const tokensPerSecond = (tokensGenerated / duration) * 1000;

    console.log(`📊 ${modelName} Results:`);
    console.log(`   Duration: ${duration}ms`);
    console.log(`   Tokens generated: ~${tokensGenerated}`);
    console.log(`   Tokens/second: ${tokensPerSecond.toFixed(2)}`);
    console.log(`   Response: "${response.text.substring(0, 100)}..."`);

    return {
      model: modelName,
      duration,
      tokensGenerated,
      tokensPerSecond,
      response: response.text,
      success: true,
    };
  } catch (error) {
    console.log(`❌ ${modelName} failed: ${error.message}`);
    return {
      model: modelName,
      success: false,
      error: error.message,
    };
  }
}

async function runBenchmarks() {
  console.log("🚀 Starting Gemma Model Benchmarks");
  console.log("===================================");

  const results = [];

  for (const model of MODELS_TO_TEST) {
    const result = await benchmarkModel(model);
    if (result) {
      results.push(result);
    }
  }

  console.log("\n📈 Benchmark Summary");
  console.log("====================");

  const successfulResults = results.filter((r) => r.success);

  if (successfulResults.length === 0) {
    console.log("❌ No models completed successfully");
    return;
  }

  // Sort by tokens per second (highest first)
  successfulResults.sort((a, b) => b.tokensPerSecond - a.tokensPerSecond);

  successfulResults.forEach((result, index) => {
    const rank = index + 1;
    const medal =
      rank === 1 ? "🥇" : rank === 2 ? "🥈" : rank === 3 ? "🥉" : "📊";
    console.log(
      `${medal} ${result.model}: ${result.tokensPerSecond.toFixed(
        2
      )} tokens/sec (${result.duration}ms)`
    );
  });

  const bestModel = successfulResults[0];
  console.log(
    `\n🎯 Recommendation: Use ${bestModel.model} for fastest inference`
  );

  // Check if it's a gemma3n model as user suspected
  if (bestModel.model.startsWith("gemma3n:")) {
    console.log("✅ As you suspected, a gemma3n model is fastest!");
  }

  return successfulResults;
}

// Run the benchmarks
runBenchmarks()
  .then((results) => {
    console.log("\n✅ Benchmarks completed");
    process.exit(0);
  })
  .catch((error) => {
    console.error("💥 Benchmark failed:", error);
    process.exit(1);
  });
