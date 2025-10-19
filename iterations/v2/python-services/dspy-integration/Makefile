# DSPy Integration Service - Makefile
#
# @author @darianrosebrook

.PHONY: help install validate test run dev clean

help:
	@echo "DSPy Integration Service - Available Commands:"
	@echo ""
	@echo "Installation:"
	@echo "  make install          - Install all dependencies (dev + prod)"
	@echo "  make install-prod     - Install production dependencies only"
	@echo "  make install-dev      - Install development dependencies only"
	@echo ""
	@echo "Testing:"
	@echo "  make test             - Run basic integration tests"
	@echo "  make test-pytest      - Run pytest suite"
	@echo "  make test-parallel    - Run tests in parallel (recommended)"
	@echo "  make test-fast        - Run fast tests only (parallel)"
	@echo "  make test-unit        - Run unit tests only"
	@echo "  make test-integration - Run integration tests only"
	@echo "  make test-coverage    - Generate coverage report"
	@echo ""
	@echo "Development:"
	@echo "  make dev              - Start development server"
	@echo "  make run              - Start production server"
	@echo "  make validate         - Validate Ollama connection"
	@echo ""
	@echo "Optimization:"
	@echo "  make optimize         - Run DSPy optimization pipeline"
	@echo "  make optimize-parallel- Run parallel optimization (multiprocessing)"
	@echo "  make benchmark        - Run performance benchmarks"
	@echo ""
	@echo "Quality:"
	@echo "  make lint             - Run code quality checks"
	@echo "  make format           - Format code with black/isort"
	@echo "  make typecheck        - Run mypy type checking"
	@echo "  make quality          - Run full quality suite"
	@echo ""
	@echo "Maintenance:"
	@echo "  make clean            - Clean cache and temp files"
	@echo ""

install:
	@echo "📦 Installing dependencies with uv..."
	uv sync --dev
	@echo "✅ Dependencies installed"

install-prod:
	@echo "📦 Installing production dependencies with uv..."
	uv sync
	@echo "✅ Production dependencies installed"

install-dev:
	@echo "📦 Installing development dependencies with uv..."
	uv sync --dev
	@echo "✅ Development dependencies installed"

validate:
	@echo "🔍 Validating Ollama connection..."
	python validate_ollama.py

test:
	@echo "🧪 Running integration tests..."
	uv run python test_integration.py

test-pytest:
	@echo "🧪 Running pytest suite..."
	uv run pytest tests/ -v --cov=. --cov-report=term-missing

test-parallel:
	@echo "🧪 Running pytest suite in parallel..."
	uv run pytest tests/ -v --cov=. --cov-report=term-missing -n auto

test-fast:
	@echo "⚡ Running fast tests only..."
	uv run pytest tests/ -v --cov=. --cov-report=term-missing -n auto -m "not slow"

test-unit:
	@echo "🧪 Running unit tests..."
	uv run pytest tests/ -v -k "unit" --cov=. --cov-report=term-missing

test-integration:
	@echo "🔗 Running integration tests..."
	uv run pytest tests/ -v -k "integration" --cov=. --cov-report=term-missing

test-coverage:
	@echo "📊 Generating coverage report..."
	uv run pytest tests/ --cov=. --cov-report=html --cov-report=term-missing
	@echo "📊 Coverage report generated at htmlcov/index.html"

run:
	@echo "🚀 Starting production server..."
	uv run uvicorn main:app --host localhost --port 8001

dev:
	@echo "🔧 Starting development server..."
	uv run uvicorn main:app --reload --host localhost --port 8001

benchmark:
	@echo "📊 Running benchmarks..."
	uv run python -m pytest benchmarks/ -v --tb=short

optimize:
	@echo "🔬 Running optimization pipeline..."
	uv run python run_optimization.py

optimize-parallel:
	@echo "🔬 Running parallel optimization pipeline..."
	uv run python parallel_optimization.py

lint:
	@echo "🔍 Running code quality checks..."
	uv run black --check .
	uv run isort --check-only .
	uv run flake8 .

format:
	@echo "💅 Formatting code..."
	uv run black .
	uv run isort .

typecheck:
	@echo "🔍 Running type checking..."
	uv run mypy .

quality:
	@echo "🏆 Running full quality suite..."
	uv run black --check .
	uv run isort --check-only .
	uv run flake8 .
	uv run mypy .

clean:
	@echo "🧹 Cleaning cache and temp files..."
	rm -rf __pycache__ .pytest_cache .coverage htmlcov
	find . -type d -name __pycache__ -exec rm -rf {} + 2>/dev/null || true
	@echo "✅ Clean complete"

