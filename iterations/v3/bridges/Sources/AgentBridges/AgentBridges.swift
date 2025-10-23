// ============================================================================
// Agent Bridges - Unified Interface
// ============================================================================
// This is an umbrella target that provides unified access to all bridge
// functionality. It imports all bridge modules to ensure they're linked
// into the final product.
//
// This target serves as the main entry point for applications that want
// to use the complete Agent Bridges functionality.
// ============================================================================

// Import all bridge modules to ensure they're linked
@_exported import Core
@_exported import Audio_STT_Whisper
@_exported import Audio_STT_SpeechFramework
@_exported import Audio_Utils
@_exported import Vision_OD_YOLO
@_exported import Vision_OCR_VisionOCR
@_exported import Vision_ImageUtils
@_exported import Text_LLM_Mistral
@_exported import Text_Tokenization
@_exported import Text_Generation_Diffusion
@_exported import System_ModelMgmt
@_exported import System_Perf

/// Agent Bridges version information
public let agentBridgesVersion = "1.0.0"

/// Agent Bridges build information
public let agentBridgesBuildInfo = """
Agent Bridges v\(agentBridgesVersion)
Unified Swift bridge architecture for Agent Agency
Supports: Audio (Whisper, Speech), Vision (YOLO, OCR), Text (Mistral, Diffusion)
FFI-ready for Rust integration
"""
