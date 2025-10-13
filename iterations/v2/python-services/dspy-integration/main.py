"""
Main FastAPI application for DSPy Integration Service

@author @darianrosebrook
"""

import os
from contextlib import asynccontextmanager
from fastapi import FastAPI, HTTPException
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel
import structlog
from dotenv import load_dotenv
import dspy
from config import (
    OLLAMA_HOST,
    OLLAMA_PRIMARY_MODEL,
    OLLAMA_FAST_MODEL,
    OLLAMA_QUALITY_MODEL,
    OLLAMA_ALTERNATIVE_MODEL,
    DEFAULT_PROVIDER,
    DSPY_LOCAL_FIRST,
    get_provider_status,
)
from ollama_lm import OllamaDSPyLM, create_ollama_clients

# Load environment variables
load_dotenv()

# Configure structured logging
logger = structlog.get_logger()


# Global DSPy clients
ollama_clients = {}
primary_lm = None
fast_lm = None
quality_lm = None
alternative_lm = None


@asynccontextmanager
async def lifespan(app: FastAPI):
    """
    Application lifespan manager.
    
    Handles startup and shutdown tasks for the DSPy service.
    """
    global ollama_clients, primary_lm, fast_lm, quality_lm, alternative_lm
    
    # Startup
    logger.info("dspy_service_starting", version="0.1.0")
    
    # Initialize DSPy with local-first configuration
    try:
        if DEFAULT_PROVIDER == "ollama":
            logger.info(
                "initializing_ollama_clients",
                host=OLLAMA_HOST,
                primary_model=OLLAMA_PRIMARY_MODEL,
            )
            
            # Create Ollama clients
            ollama_clients = create_ollama_clients(host=OLLAMA_HOST)
            
            # Assign to global variables for easy access
            primary_lm = ollama_clients.get("primary")
            fast_lm = ollama_clients.get("fast")
            quality_lm = ollama_clients.get("quality")
            alternative_lm = ollama_clients.get("alternative")
            
            # Configure DSPy to use primary model by default
            if primary_lm and primary_lm.is_available():
                dspy.settings.configure(lm=primary_lm)
                logger.info(
                    "dspy_configured_with_ollama",
                    model=OLLAMA_PRIMARY_MODEL,
                )
            else:
                logger.error(
                    "ollama_primary_model_unavailable",
                    model=OLLAMA_PRIMARY_MODEL,
                    message="Make sure Ollama is running and model is pulled",
                )
            
        else:
            logger.warning(
                "using_paid_provider",
                provider=DEFAULT_PROVIDER,
                local_first=DSPY_LOCAL_FIRST,
            )
        
        # Log configuration status
        status = get_provider_status()
        logger.info("dspy_configuration_status", **status)
        
    except Exception as error:
        logger.error("dspy_initialization_failed", error=str(error))
        # Don't raise - allow service to start even if Ollama isn't available
        logger.warning("service_starting_without_ollama")
    
    yield
    
    # Shutdown
    logger.info("dspy_service_shutting_down")


# Create FastAPI application
app = FastAPI(
    title="DSPy Integration Service",
    description="DSPy-powered prompt optimization for Agent Agency V2",
    version="0.1.0",
    lifespan=lifespan
)

# Configure CORS for TypeScript application
app.add_middleware(
    CORSMiddleware,
    allow_origins=[
        "http://localhost:3000",
        "http://localhost:8000",
    ],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)


# Request/Response Models
class HealthResponse(BaseModel):
    """Health check response."""
    status: str
    version: str
    dspy_configured: bool
    provider: str = "ollama"
    local_first: bool = True
    ollama_configured: bool = False


class RubricOptimizationRequest(BaseModel):
    """Request model for rubric optimization."""
    task_context: str
    agent_output: str
    evaluation_criteria: str


class RubricOptimizationResponse(BaseModel):
    """Response model for rubric optimization."""
    reward_score: float
    reasoning: str
    improvement_suggestions: str
    metadata: dict


class JudgeEvaluationRequest(BaseModel):
    """Request model for judge evaluation."""
    judge_type: str  # relevance, faithfulness, minimality, safety
    artifact: str
    ground_truth: str
    context: str


class JudgeEvaluationResponse(BaseModel):
    """Response model for judge evaluation."""
    judgment: str
    confidence: float
    reasoning: str
    metadata: dict


class SignatureOptimizationRequest(BaseModel):
    """Request model for signature optimization."""
    signature_id: str
    eval_data: list
    optimizer: str  # MIPROv2, SIMBA, etc.


class SignatureOptimizationResponse(BaseModel):
    """Response model for signature optimization."""
    optimized_signature_id: str
    improvement_metrics: dict
    metadata: dict


# API Routes
@app.get("/health", response_model=HealthResponse)
async def health_check():
    """
    Health check endpoint.

    Returns:
        HealthResponse: Service health status
    """
    provider_status = get_provider_status()

    return HealthResponse(
        status="healthy",
        version="0.1.0",
        dspy_configured=True,
        provider=DEFAULT_PROVIDER,
        local_first=DSPY_LOCAL_FIRST,
        ollama_configured=provider_status["ollama"]["primary_model"] is not None,
    )


@app.post("/api/v1/rubric/optimize", response_model=RubricOptimizationResponse)
async def optimize_rubric(request: RubricOptimizationRequest):
    """
    Optimize rubric computation using DSPy.

    Args:
        request: Rubric optimization request

    Returns:
        RubricOptimizationResponse: Optimized rubric evaluation

    Raises:
        HTTPException: If optimization fails
    """
    logger.info("rubric_optimization_requested",
                task_context=request.task_context[:100])

    # TODO: Implement DSPy rubric optimization
    raise HTTPException(
        status_code=501,
        detail="Rubric optimization not yet implemented"
    )


@app.post("/api/v1/judge/evaluate", response_model=JudgeEvaluationResponse)
async def evaluate_with_judge(request: JudgeEvaluationRequest):
    """
    Evaluate artifact using self-improving model judge.

    Args:
        request: Judge evaluation request

    Returns:
        JudgeEvaluationResponse: Judge evaluation result

    Raises:
        HTTPException: If evaluation fails
    """
    logger.info("judge_evaluation_requested",
                judge_type=request.judge_type)

    # TODO: Implement DSPy model judge
    raise HTTPException(
        status_code=501,
        detail="Judge evaluation not yet implemented"
    )


@app.post("/api/v1/optimize/signature", response_model=SignatureOptimizationResponse)
async def optimize_signature(request: SignatureOptimizationRequest):
    """
    Optimize DSPy signature using evaluation data.

    Args:
        request: Signature optimization request

    Returns:
        SignatureOptimizationResponse: Optimization results

    Raises:
        HTTPException: If optimization fails
    """
    logger.info("signature_optimization_requested",
                signature_id=request.signature_id,
                optimizer=request.optimizer)

    # TODO: Implement signature optimization
    raise HTTPException(
        status_code=501,
        detail="Signature optimization not yet implemented"
    )


if __name__ == "__main__":
    import uvicorn

    port = int(os.getenv("DSPY_SERVICE_PORT", "8001"))
    host = os.getenv("DSPY_SERVICE_HOST", "localhost")

    uvicorn.run(
        "main:app",
        host=host,
        port=port,
        reload=True,
        log_level="info"
    )
