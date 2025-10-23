#!/usr/bin/env python3
"""
MCP Documentation Quality Server

Provides documentation quality validation as an MCP tool for autonomous agents.
Integrates with the existing MCP ecosystem to provide documentation quality
validation capabilities to AI models and agents.
"""

import asyncio
import json
import subprocess
import sys
from pathlib import Path
from typing import Dict, Any, List, Optional
import tempfile
import os

class DocQualityMCPServer:
    def __init__(self):
        self.project_root = Path(__file__).parent.parent
        self.linter_path = self.project_root / "scripts" / "doc-quality-linter.py"
        
    async def handle_initialize(self, request: Dict[str, Any]) -> Dict[str, Any]:
        """Handle MCP initialization request."""
        return {
            "jsonrpc": "2.0",
            "id": request.get("id"),
            "result": {
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {}
                },
                "serverInfo": {
                    "name": "doc-quality-validator",
                    "version": "1.0.0"
                }
            }
        }
    
    async def handle_tools_list(self, request: Dict[str, Any]) -> Dict[str, Any]:
        """Handle tools list request."""
        return {
            "jsonrpc": "2.0",
            "id": request.get("id"),
            "result": {
                "tools": [
                    {
                        "name": "doc_quality_validator",
                        "description": "Validates documentation quality against engineering standards and prevents problematic content",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "content": {
                                    "type": "string",
                                    "description": "Documentation content to validate"
                                },
                                "content_type": {
                                    "type": "string",
                                    "enum": ["markdown", "text", "rst", "adoc"],
                                    "description": "Type of documentation content"
                                },
                                "file_path": {
                                    "type": "string",
                                    "description": "Path to the documentation file (optional)"
                                },
                                "validation_level": {
                                    "type": "string",
                                    "enum": ["strict", "moderate", "lenient"],
                                    "default": "moderate",
                                    "description": "Validation strictness level"
                                },
                                "include_suggestions": {
                                    "type": "boolean",
                                    "default": True,
                                    "description": "Include suggested fixes for issues"
                                }
                            },
                            "required": ["content", "content_type"]
                        }
                    }
                ]
            }
        }
    
    async def handle_tools_call(self, request: Dict[str, Any]) -> Dict[str, Any]:
        """Handle tool call request."""
        params = request.get("params", {})
        tool_name = params.get("name")
        arguments = params.get("arguments", {})
        
        if tool_name == "doc_quality_validator":
            result = await self.validate_documentation_quality(arguments)
        else:
            result = {
                "error": f"Unknown tool: {tool_name}",
                "content": []
            }
        
        return {
            "jsonrpc": "2.0",
            "id": request.get("id"),
            "result": {
                "content": [
                    {
                        "type": "text",
                        "text": json.dumps(result, indent=2)
                    }
                ]
            }
        }
    
    async def validate_documentation_quality(self, args: Dict[str, Any]) -> Dict[str, Any]:
        """Validate documentation quality using the linter."""
        try:
            content = args.get("content", "")
            content_type = args.get("content_type", "markdown")
            file_path = args.get("file_path", "")
            validation_level = args.get("validation_level", "moderate")
            include_suggestions = args.get("include_suggestions", True)
            
            # Create temporary file for content
            with tempfile.NamedTemporaryFile(mode='w', suffix=f'.{content_type}', delete=False) as temp_file:
                temp_file.write(content)
                temp_file_path = temp_file.name
            
            try:
                # Run the documentation quality linter
                cmd = [
                    sys.executable,
                    str(self.linter_path),
                    "--path", temp_file_path,
                    "--format", "json"
                ]
                
                # Add validation level if specified
                if validation_level != "moderate":
                    cmd.extend(["--validation-level", validation_level])
                
                # Run the linter
                result = subprocess.run(
                    cmd,
                    capture_output=True,
                    text=True,
                    timeout=30
                )
                
                if result.returncode != 0:
                    return {
                        "error": f"Linter failed: {result.stderr}",
                        "validation_id": f"val_{hash(content)}",
                        "quality_score": 0.0,
                        "issues": [],
                        "metrics": {},
                        "recommendations": ["Fix linter errors before validation"]
                    }
                
                # Parse the JSON output
                try:
                    linter_output = json.loads(result.stdout)
                except json.JSONDecodeError:
                    # Fallback to text parsing if JSON fails
                    linter_output = self._parse_text_output(result.stdout)
                
                # Calculate quality score
                issues = linter_output.get("issues", [])
                quality_score = self._calculate_quality_score(issues, validation_level)
                
                # Generate metrics
                metrics = self._generate_metrics(issues)
                
                # Generate recommendations
                recommendations = self._generate_recommendations(issues, quality_score)
                
                return {
                    "validation_id": f"val_{hash(content)}",
                    "quality_score": quality_score,
                    "issues": issues,
                    "metrics": metrics,
                    "recommendations": recommendations
                }
                
            finally:
                # Clean up temporary file
                os.unlink(temp_file_path)
                
        except Exception as e:
            return {
                "error": f"Validation failed: {str(e)}",
                "validation_id": f"val_{hash(content)}",
                "quality_score": 0.0,
                "issues": [],
                "metrics": {},
                "recommendations": ["Fix validation errors and retry"]
            }
    
    def _parse_text_output(self, text_output: str) -> Dict[str, Any]:
        """Parse text output from linter when JSON fails."""
        issues = []
        lines = text_output.split('\n')
        
        for line in lines:
            if ' - ' in line and ('ERROR' in line or 'WARNING' in line or 'INFO' in line):
                parts = line.split(' - ', 1)
                if len(parts) == 2:
                    file_path = parts[0].strip()
                    message = parts[1].strip()
                    
                    severity = "info"
                    if "ERROR" in message:
                        severity = "error"
                    elif "WARNING" in message:
                        severity = "warning"
                    
                    issues.append({
                        "severity": severity,
                        "rule_id": "unknown",
                        "message": message,
                        "line_number": 0,
                        "suggested_fix": "Review and fix the issue"
                    })
        
        return {"issues": issues}
    
    def _calculate_quality_score(self, issues: List[Dict[str, Any]], validation_level: str) -> float:
        """Calculate quality score based on issues and validation level."""
        if not issues:
            return 1.0
        
        # Count issues by severity
        error_count = sum(1 for issue in issues if issue.get("severity") == "error")
        warning_count = sum(1 for issue in issues if issue.get("severity") == "warning")
        info_count = sum(1 for issue in issues if issue.get("severity") == "info")
        
        # Calculate base score
        total_issues = len(issues)
        base_score = max(0.0, 1.0 - (total_issues * 0.1))
        
        # Apply severity penalties
        error_penalty = error_count * 0.2
        warning_penalty = warning_count * 0.1
        info_penalty = info_count * 0.05
        
        # Apply validation level multiplier
        level_multiplier = {
            "strict": 1.0,
            "moderate": 0.8,
            "lenient": 0.6
        }.get(validation_level, 0.8)
        
        final_score = max(0.0, base_score - error_penalty - warning_penalty - info_penalty)
        return min(1.0, final_score * level_multiplier)
    
    def _generate_metrics(self, issues: List[Dict[str, Any]]) -> Dict[str, int]:
        """Generate quality metrics from issues."""
        metrics = {
            "superiority_claims": 0,
            "unfounded_achievements": 0,
            "marketing_language": 0,
            "temporal_docs": 0,
            "emoji_usage": 0
        }
        
        for issue in issues:
            rule_id = issue.get("rule_id", "")
            if "SUPERIORITY_CLAIM" in rule_id:
                metrics["superiority_claims"] += 1
            elif "UNFOUNDED_ACHIEVEMENT" in rule_id:
                metrics["unfounded_achievements"] += 1
            elif "MARKETING_LANGUAGE" in rule_id:
                metrics["marketing_language"] += 1
            elif "TEMPORAL_DOC" in rule_id:
                metrics["temporal_docs"] += 1
            elif "EMOJI_USAGE" in rule_id:
                metrics["emoji_usage"] += 1
        
        return metrics
    
    def _generate_recommendations(self, issues: List[Dict[str, Any]], quality_score: float) -> List[str]:
        """Generate recommendations based on issues and quality score."""
        recommendations = []
        
        if quality_score < 0.5:
            recommendations.append("Documentation quality is very low. Consider a complete rewrite focusing on engineering-grade content.")
        elif quality_score < 0.8:
            recommendations.append("Documentation quality needs improvement. Address the identified issues.")
        
        # Add specific recommendations based on issue types
        issue_types = set(issue.get("rule_id", "") for issue in issues)
        
        if "SUPERIORITY_CLAIM" in issue_types:
            recommendations.append("Remove superiority claims and marketing language. Focus on technical capabilities.")
        
        if "UNFOUNDED_ACHIEVEMENT" in issue_types:
            recommendations.append("Verify all achievement claims with evidence or use more accurate language.")
        
        if "TEMPORAL_DOC" in issue_types:
            recommendations.append("Move temporal documentation to appropriate archive directories.")
        
        if "EMOJI_USAGE" in issue_types:
            recommendations.append("Remove emojis or use only approved emojis (⚠️, ✅, 🚫).")
        
        if not recommendations:
            recommendations.append("Documentation quality is good. Continue maintaining engineering-grade standards.")
        
        return recommendations
    
    async def handle_request(self, request: Dict[str, Any]) -> Dict[str, Any]:
        """Handle incoming MCP requests."""
        method = request.get("method")
        
        if method == "initialize":
            return await self.handle_initialize(request)
        elif method == "tools/list":
            return await self.handle_tools_list(request)
        elif method == "tools/call":
            return await self.handle_tools_call(request)
        else:
            return {
                "jsonrpc": "2.0",
                "id": request.get("id"),
                "error": {
                    "code": -32601,
                    "message": f"Method not found: {method}"
                }
            }

async def main():
    """Main entry point for the MCP server."""
    server = DocQualityMCPServer()
    
    # Read from stdin and write to stdout
    while True:
        try:
            line = sys.stdin.readline()
            if not line:
                break
            
            request = json.loads(line.strip())
            response = await server.handle_request(request)
            print(json.dumps(response))
            sys.stdout.flush()
            
        except json.JSONDecodeError:
            continue
        except Exception as e:
            error_response = {
                "jsonrpc": "2.0",
                "id": None,
                "error": {
                    "code": -32603,
                    "message": f"Internal error: {str(e)}"
                }
            }
            print(json.dumps(error_response))
            sys.stdout.flush()

if __name__ == "__main__":
    asyncio.run(main())
