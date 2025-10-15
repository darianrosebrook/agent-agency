# DSPy Integration Service - Makefile
#
# @author @darianrosebrook

.PHONY: help install validate test run dev clean

help:
	@echo "DSPy Integration Service - Available Commands:"
	@echo ""
	@echo "  make install   - Install dependencies"
	@echo "  make validate  - Validate Ollama connection"
	@echo "  make test      - Run integration tests"
	@echo "  make run       - Start production server"
	@echo "  make dev       - Start development server"
	@echo "  make clean     - Clean cache and temp files"
	@echo ""

install:
	@echo "📦 Installing dependencies..."
	pip install -r requirements.txt
	@echo "✅ Dependencies installed"

validate:
	@echo "🔍 Validating Ollama connection..."
	python validate_ollama.py

test:
	@echo "🧪 Running integration tests..."
	python test_integration.py

test-pytest:
	@echo "🧪 Running pytest suite..."
	pytest tests/ -v --cov=. --cov-report=term-missing

run:
	@echo "🚀 Starting production server..."
	uvicorn main:app --host localhost --port 8001

dev:
	@echo "🔧 Starting development server..."
	uvicorn main:app --reload --host localhost --port 8001

clean:
	@echo "🧹 Cleaning cache and temp files..."
	rm -rf __pycache__ .pytest_cache .coverage htmlcov
	find . -type d -name __pycache__ -exec rm -rf {} + 2>/dev/null || true
	@echo "✅ Clean complete"

