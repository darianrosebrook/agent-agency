#!/bin/bash

echo "🧠 Agent Agency - Core ML Model Enumeration Test"
echo "================================================"

MODEL_PATH="${COREML_MODELS_PATH:-/Users/darianrosebrook/Desktop/Projects/agent-agency/models/coreml}"

echo "📁 Model path: $MODEL_PATH"
echo

# Check if directory exists
if [ ! -d "$MODEL_PATH" ]; then
    echo "❌ Model directory not found: $MODEL_PATH"
    echo "💡 Set COREML_MODELS_PATH environment variable or ensure models are at the default location"
    exit 1
fi

echo "🔍 Checking for Core ML models..."
echo

# Check for FastViT vision model
if [ -f "$MODEL_PATH/fastvit/FastViTT8F16.mlpackage.mlmodelc/coremldata.bin" ]; then
    echo "✅ FastViT Vision Model: Found"
    echo "   📁 Path: $MODEL_PATH/fastvit/FastViTT8F16.mlpackage.mlmodelc"
    echo "   🎯 Type: Vision Classification"
    echo "   🚀 ANE Optimized: Yes"
    FASTVIT_FOUND=true
else
    echo "❌ FastViT Vision Model: Not found"
    FASTVIT_FOUND=false
fi
echo

# Check for Mistral language model
if [ -f "$MODEL_PATH/mistral/StatefulMistral7BInstructFP16.mlpackage.mlmodelc/coremldata.bin" ]; then
    echo "✅ Mistral Language Model: Found"
    echo "   📁 Path: $MODEL_PATH/mistral/StatefulMistral7BInstructFP16.mlpackage.mlmodelc"
    echo "   🎯 Type: Large Language Model (7B parameters)"
    echo "   🚀 ANE Optimized: Yes"
    MISTRAL_FOUND=true
else
    echo "❌ Mistral Language Model: Not found"
    MISTRAL_FOUND=false
fi
echo

# Check for Whisper speech model
if [ -f "$MODEL_PATH/whisper/ggml-base.en-encoder.mlmodelc/coremldata.bin" ]; then
    echo "✅ Whisper Speech Model: Found"
    echo "   📁 Path: $MODEL_PATH/whisper/ggml-base.en-encoder.mlmodelc"
    echo "   🎯 Type: Speech-to-Text"
    echo "   🚀 ANE Optimized: Yes"
    WHISPER_FOUND=true
else
    echo "❌ Whisper Speech Model: Not found"
    WHISPER_FOUND=false
fi
echo

# Check for YOLO detection model
if [ -f "$MODEL_PATH/yolov3/YOLOv3.mlmodel.mlmodelc/coremldata.bin" ]; then
    echo "✅ YOLO Detection Model: Found"
    echo "   📁 Path: $MODEL_PATH/yolov3/YOLOv3.mlmodel.mlmodelc"
    echo "   🎯 Type: Object Detection"
    echo "   🚀 ANE Optimized: Yes"
    YOLO_FOUND=true
else
    echo "❌ YOLO Detection Model: Not found"
    YOLO_FOUND=false
fi
echo

# Summary
MODEL_COUNT=0
[ "$FASTVIT_FOUND" = true ] && ((MODEL_COUNT++))
[ "$MISTRAL_FOUND" = true ] && ((MODEL_COUNT++))
[ "$WHISPER_FOUND" = true ] && ((MODEL_COUNT++))
[ "$YOLO_FOUND" = true ] && ((MODEL_COUNT++))

echo "📊 Test Results Summary:"
echo "========================="
echo "📦 Total Models Found: $MODEL_COUNT/4"
echo "🎯 Vision Models: $([ "$FASTVIT_FOUND" = true ] && echo '1' || echo '0')/1"
echo "💬 Language Models: $([ "$MISTRAL_FOUND" = true ] && echo '1' || echo '0')/1"
echo "🎤 Speech Models: $([ "$WHISPER_FOUND" = true ] && echo '1' || echo '0')/1"
echo "🔍 Detection Models: $([ "$YOLO_FOUND" = true ] && echo '1' || echo '0')/1"
echo

# Check ANE availability (macOS with Apple Silicon)
if [[ "$OSTYPE" == "darwin"* ]] && [[ "$(uname -m)" == "arm64" ]]; then
    echo "🍎 ANE Acceleration: ✅ Available (Apple Silicon detected)"
    ANE_AVAILABLE=true
else
    echo "🍎 ANE Acceleration: ❌ Not Available (Intel macOS or non-macOS)"
    ANE_AVAILABLE=false
fi
echo

# Final assessment
if [ "$MODEL_COUNT" -eq 4 ]; then
    echo "🎉 SUCCESS: All Core ML models found and ready!"
    echo "💡 Ready for Phase 3B: Actual inference testing and ANE speedup measurement"
    echo
    echo "🚀 Next Steps:"
    echo "   1. Run actual inference tests to measure ANE speedup (target: 2.8x)"
    echo "   2. Test dispatch rate optimization (target: 70% of peak)"
    echo "   3. Validate end-to-end pipeline performance"
    exit 0
elif [ "$MODEL_COUNT" -gt 0 ]; then
    echo "⚠️  PARTIAL: Some Core ML models found ($MODEL_COUNT/4)"
    echo "💡 Models are detected but integration may be incomplete"
    exit 1
else
    echo "❌ FAILURE: No Core ML models found"
    echo "💡 Check COREML_MODELS_PATH environment variable"
    echo "   Expected path: /Users/darianrosebrook/Desktop/Projects/agent-agency/models/coreml"
    exit 1
fi
