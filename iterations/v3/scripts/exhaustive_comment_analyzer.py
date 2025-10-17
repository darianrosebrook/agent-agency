#!/usr/bin/env python3
"""
Exhaustive Comment Analysis Script

@description: Comprehensive analysis of ALL comments in project files to discover
hidden TODO patterns, incomplete implementations, and technical debt indicators.
This script goes beyond the standard todo analyzer to capture every possible
indicator of incomplete work across all supported languages.

@author: @darianrosebrook
@date: 2025-01-27
@version: 1.0.0
"""

import os
import re
import json
import time
from pathlib import Path
from collections import defaultdict, Counter
from typing import Dict, List, Set, Tuple, Optional, Any
from dataclasses import dataclass, asdict


@dataclass
class CommentAnalysis:
    """Data class for comment analysis results."""
    file_path: str
    line_number: int
    language: str
    comment_text: str
    confidence_score: float
    pattern_category: str
    matched_patterns: List[str]
    context_score: float
    is_explicit_todo: bool
    is_hidden_todo: bool
    is_placeholder: bool
    is_temporary: bool
    is_incomplete: bool
    is_technical_debt: bool


class ExhaustiveCommentAnalyzer:
    """Comprehensive comment analyzer that captures all possible TODO indicators."""
    
    def __init__(self, root_dir: str):
        self.root_dir = Path(root_dir)
        self.start_time = time.time()
        
        # Enhanced language patterns with more extensions
        self.language_patterns = {
            'rust': {
                'extensions': ['.rs'],
                'single_line': r'^\s*//',
                'multi_line_start': r'^\s*/\*',
                'multi_line_end': r'\*/',
                'doc_comment': r'^\s*///',
                'doc_block': r'^\s*//!',
            },
            'javascript': {
                'extensions': ['.js', '.mjs', '.cjs', '.jsx'],
                'single_line': r'^\s*//',
                'multi_line_start': r'^\s*/\*',
                'multi_line_end': r'\*/',
                'doc_comment': r'^\s*/\*\*',
                'doc_block': None,
            },
            'typescript': {
                'extensions': ['.ts', '.tsx', '.mts', '.cts', '.d.ts'],
                'single_line': r'^\s*//',
                'multi_line_start': r'^\s*/\*',
                'multi_line_end': r'\*/',
                'doc_comment': r'^\s*/\*\*',
                'doc_block': None,
            },
            'python': {
                'extensions': ['.py', '.pyi', '.pyw'],
                'single_line': r'^\s*#',
                'multi_line_start': r'^\s*"""',
                'multi_line_end': r'"""',
                'doc_comment': r'^\s*"""',
                'doc_block': None,
            },
            'go': {
                'extensions': ['.go'],
                'single_line': r'^\s*//',
                'multi_line_start': r'^\s*/\*',
                'multi_line_end': r'\*/',
                'doc_comment': r'^\s*//\s+[A-Z]',
                'doc_block': None,
            },
            'java': {
                'extensions': ['.java'],
                'single_line': r'^\s*//',
                'multi_line_start': r'^\s*/\*',
                'multi_line_end': r'\*/',
                'doc_comment': r'^\s*/\*\*',
                'doc_block': None,
            },
            'csharp': {
                'extensions': ['.cs'],
                'single_line': r'^\s*//',
                'multi_line_start': r'^\s*/\*',
                'multi_line_end': r'\*/',
                'doc_comment': r'^\s*///',
                'doc_block': None,
            },
            'cpp': {
                'extensions': ['.cpp', '.cc', '.cxx', '.c++', '.hpp', '.h', '.hxx', '.h++'],
                'single_line': r'^\s*//',
                'multi_line_start': r'^\s*/\*',
                'multi_line_end': r'\*/',
                'doc_comment': r'^\s*///',
                'doc_block': None,
            },
            'c': {
                'extensions': ['.c', '.h'],
                'single_line': r'^\s*//',
                'multi_line_start': r'^\s*/\*',
                'multi_line_end': r'\*/',
                'doc_comment': r'^\s*/\*\*',
                'doc_block': None,
            },
            'php': {
                'extensions': ['.php', '.phtml'],
                'single_line': r'^\s*//',
                'multi_line_start': r'^\s*/\*',
                'multi_line_end': r'\*/',
                'doc_comment': r'^\s*/\*\*',
                'doc_block': None,
            },
            'ruby': {
                'extensions': ['.rb', '.rbw'],
                'single_line': r'^\s*#',
                'multi_line_start': r'^\s*=begin',
                'multi_line_end': r'=end',
                'doc_comment': r'^\s*#',
                'doc_block': None,
            },
            'shell': {
                'extensions': ['.sh', '.bash', '.zsh', '.fish', '.ksh'],
                'single_line': r'^\s*#',
                'multi_line_start': None,
                'multi_line_end': None,
                'doc_comment': None,
                'doc_block': None,
            },
            'yaml': {
                'extensions': ['.yaml', '.yml'],
                'single_line': r'^\s*#',
                'multi_line_start': None,
                'multi_line_end': None,
                'doc_comment': None,
                'doc_block': None,
            },
            'toml': {
                'extensions': ['.toml'],
                'single_line': r'^\s*#',
                'multi_line_start': None,
                'multi_line_end': None,
                'doc_comment': None,
                'doc_block': None,
            },
            'json': {
                'extensions': ['.json'],
                'single_line': None,
                'multi_line_start': None,
                'multi_line_end': None,
                'doc_comment': None,
                'doc_block': None,
            },
            'markdown': {
                'extensions': ['.md', '.markdown', '.mdown', '.mkd'],
                'single_line': r'^\s*<!--',
                'multi_line_start': r'^\s*<!--',
                'multi_line_end': r'-->',
                'doc_comment': None,
                'doc_block': None,
            },
            'sql': {
                'extensions': ['.sql'],
                'single_line': r'^\s*--',
                'multi_line_start': r'^\s*/\*',
                'multi_line_end': r'\*/',
                'doc_comment': None,
                'doc_block': None,
            },
            'dockerfile': {
                'extensions': ['Dockerfile', 'dockerfile'],
                'single_line': r'^\s*#',
                'multi_line_start': None,
                'multi_line_end': None,
                'doc_comment': None,
                'doc_block': None,
            },
        }

        # Comprehensive file ignore patterns
        self.ignored_patterns = [
            # Build artifacts
            r'\btarget\b', r'\bbuild\b', r'\bout\b', r'\bdist\b', r'\bbin\b',
            r'\.next\b', r'generated\.', r'bindgen\.', r'private\.',
            r'mime_types_generated\.', r'named_entities\.',
            r'ascii_case_insensitive_html_attributes\.',
            
            # Package management
            r'\bnode_modules\b', r'package-lock\.json$', r'yarn\.lock$',
            r'pnpm-lock\.yaml$', r'\bvenv\b', r'Cargo\.lock$',
            
            # Version control and IDE
            r'\.git\b', r'\.github\b', r'\.vscode\b', r'\.idea\b',
            r'\.DS_Store$', r'\._', r'\.Spotlight-V100$',
            
            # Test files
            r'\btest\b', r'\btests\b', r'_test\.', r'_tests\.',
            r'\.test\.', r'\.spec\.', r'\.specs\.',
            
            # Documentation and examples
            r'\bdocs\b', r'\bexamples\b', r'\bdoc\b', r'\bexample\b',
            
            # Temporary and cache
            r'\bcache\b', r'\btmp\b', r'\btemp\b', r'\.tmp$', r'\.temp$',
            r'\.cache$', r'__pycache__',
            
            # Build artifacts
            r'\.rlib$', r'\.rmeta$', r'\.d$', r'\.pdb$', r'\.o$', r'\.obj$',
            r'\.exe$', r'\.dll$', r'\.so$', r'\.dylib$', r'\.pyc$', r'\.pyo$',
            r'\.class$', r'\.jar$', r'\.war$', r'\.ear$',
            r'\.min\.js$', r'\.min\.css$', r'\.bundle\.', r'\.chunk\.', r'\.map$',
            
            # Configuration
            r'\.env\.local$', r'\.env\.production$', r'\.env\.development$',
            r'config\.local\.', r'config\.prod\.', r'config\.dev\.',
        ]

        # Explicit TODO patterns (highest priority)
        self.explicit_patterns = {
            'explicit_todos': [
                r'\bTODO\b.*?:',
                r'\bFIXME\b.*?:',
                r'\bHACK\b.*?:',
                r'\bXXX\b.*?:',
                r'\bBUG\b.*?:',
                r'\bNOTE\b.*?:.*?(fix|implement|complete|add|replace)',
                r'\bWARNING\b.*?:.*?(fix|implement|complete|add|replace)',
                r'\bTEMP\b.*?:.*?(implement|fix|replace|complete|add)',
                r'\bTEMPORARY\b.*?:.*?(implement|fix|replace|complete|add)',
                r'\bDEPRECATED\b.*?:.*?(remove|replace|update)',
            ]
        }

        # Comprehensive hidden TODO patterns
        self.hidden_patterns = {
            'incomplete_implementation': [
                r'\bnot\s+yet\s+implemented\b',
                r'\bmissing\s+implementation\b',
                r'\bincomplete\s+implementation\b',
                r'\bpartial\s+implementation\b',
                r'\bunimplemented\b',
                r'\bnot\s+done\b',
                r'\bpending\s+implementation\b',
                r'\bto\s+be\s+implemented\b',
                r'\bwill\s+be\s+implemented\b',
                r'\bshould\s+be\s+implemented\b',
                r'\bmust\s+be\s+implemented\b',
                r'\bneeds\s+implementation\b',
                r'\brequires\s+implementation\b',
                r'\bawaiting\s+implementation\b',
                r'\bwaiting\s+for\s+implementation\b',
            ],
            
            'placeholder_code': [
                r'\bplaceholder\s+(code|implementation|function|value|data|content)\b',
                r'\bstub\s+(implementation|function|method|class)\b',
                r'\bdummy\s+(implementation|function|data|value|content)\b',
                r'\bfake\s+(implementation|function|data|value|content)\b',
                r'\bexample\s+(implementation|function|data|value|content)\b',
                r'\bdemo\s+(implementation|function|data|value|content)\b',
                r'\bsample\s+(implementation|function|data|value|content)\b',
                r'\btemplate\s+(implementation|function|data|value|content)\b',
                r'\bmock\s+(implementation|function|data|value|content)\b',
                r'\btest\s+(implementation|function|data|value|content)\b',
                r'\bdefault\s+(implementation|function|data|value|content)\b',
                r'\bbasic\s+(implementation|function|data|value|content)\b',
                r'\bsimple\s+(implementation|function|data|value|content)\b',
                r'\bminimal\s+(implementation|function|data|value|content)\b',
                r'\bnaive\s+(implementation|function|data|value|content)\b',
                r'\brough\s+(implementation|function|data|value|content)\b',
                r'\bcrude\s+(implementation|function|data|value|content)\b',
            ],
            
            'temporary_solutions': [
                r'\btemporary\s+(solution|fix|workaround|implementation|code)\b',
                r'\btemp\s+(solution|fix|workaround|implementation|code)\b',
                r'\bquick\s+(fix|hack|solution|implementation|workaround)\b',
                r'\bworkaround\b.*?(fix|solution|implement)',
                r'\bhack\b.*?(fix|solution|implement)',
                r'\bpatch\b.*?(fix|solution|implement)',
                r'\bbypass\b.*?(fix|solution|implement)',
                r'\bkludge\b',
                r'\bmonkey\s+patch\b',
                r'\bdirty\s+(fix|solution|implementation)\b',
                r'\bcheap\s+(fix|solution|implementation)\b',
                r'\bexpedient\s+(fix|solution|implementation)\b',
            ],
            
            'hardcoded_values': [
                r'\bhardcoded\s+(value|string|number|constant|config)\b',
                r'\bhard-coded\s+(value|string|number|constant|config)\b',
                r'\bmagic\s+(number|string|value|constant)\b',
                r'\bconstant\s+value\b.*?(replace|change|make\s+configurable)',
                r'\bdefault\s+value\b.*?(replace|change|make\s+configurable)',
                r'\bstatic\s+value\b.*?(replace|change|make\s+configurable)',
                r'\bfixed\s+value\b.*?(replace|change|make\s+configurable)',
                r'\bliteral\s+value\b.*?(replace|change|make\s+configurable)',
            ],
            
            'future_improvements': [
                r'\bin\s+production\b.*?(implement|add|fix|complete)',
                r'\bin\s+a\s+real\s+implementation\b',
                r'\beventually\b.*?(implement|add|fix|complete)',
                r'\blater\b.*?(implement|add|fix|complete)',
                r'\bshould\s+be\b.*?(implemented|added|fixed|completed)',
                r'\bwould\s+be\b.*?(implemented|added|fixed|completed)',
                r'\bcould\s+be\b.*?(implemented|added|fixed|completed)',
                r'\bwill\s+be\b.*?(implemented|added|fixed|completed)',
                r'\bsomeday\b.*?(implement|add|fix|complete)',
                r'\bwhen\s+we\s+have\s+time\b',
                r'\bwhen\s+time\s+permits\b',
                r'\bin\s+the\s+future\b.*?(implement|add|fix|complete)',
                r'\bfuture\s+enhancement\b',
                r'\bfuture\s+improvement\b',
                r'\bfuture\s+work\b',
            ],
            
            'error_handling': [
                r'\bproper\s+error\s+handling\b.*?(implement|add|fix)',
                r'\berror\s+handling\b.*?(missing|incomplete|needs)',
                r'\bexception\s+handling\b.*?(missing|incomplete|needs)',
                r'\bfailure\s+handling\b.*?(missing|incomplete|needs)',
                r'\bgraceful\s+degradation\b.*?(implement|add|fix)',
                r'\bfallback\s+mechanism\b.*?(implement|add|fix)',
                r'\brecovery\s+mechanism\b.*?(implement|add|fix)',
            ],
            
            'performance_issues': [
                r'\bperformance\s+(issue|problem|bottleneck)\b.*?(fix|optimize)',
                r'\boptimization\b.*?(needed|required|missing)',
                r'\befficiency\b.*?(improve|optimize|fix)',
                r'\bmemory\s+leak\b.*?(fix|investigate)',
                r'\bresource\s+leak\b.*?(fix|investigate)',
                r'\bslow\s+(implementation|code|function)\b.*?(optimize|improve)',
                r'\binefficient\b.*?(implementation|code|function)',
            ],
            
            'security_concerns': [
                r'\bsecurity\s+(issue|vulnerability|concern)\b.*?(fix|address)',
                r'\bvulnerability\b.*?(fix|address|patch)',
                r'\bauthentication\b.*?(implement|add|fix)',
                r'\bauthorization\b.*?(implement|add|fix)',
                r'\binput\s+validation\b.*?(implement|add|fix)',
                r'\bsanitization\b.*?(implement|add|fix)',
                r'\bencryption\b.*?(implement|add|fix)',
            ],
            
            'testing_debt': [
                r'\btest\s+(missing|incomplete|needed|required)\b',
                r'\btesting\s+(missing|incomplete|needed|required)\b',
                r'\bunit\s+test\b.*?(missing|incomplete|needed|required)',
                r'\bintegration\s+test\b.*?(missing|incomplete|needed|required)',
                r'\bcoverage\b.*?(missing|incomplete|needed|required)',
                r'\bassertion\b.*?(missing|incomplete|needed|required)',
                r'\bvalidation\b.*?(missing|incomplete|needed|required)',
            ],
            
            'documentation_debt': [
                r'\bdocumentation\b.*?(missing|incomplete|needed|required)',
                r'\bdoc\b.*?(missing|incomplete|needed|required)',
                r'\bcomment\b.*?(missing|incomplete|needed|required)',
                r'\bexplanation\b.*?(missing|incomplete|needed|required)',
                r'\bdescription\b.*?(missing|incomplete|needed|required)',
                r'\bexample\b.*?(missing|incomplete|needed|required)',
                r'\busage\b.*?(missing|incomplete|needed|required)',
            ],
            
            'refactoring_debt': [
                r'\brefactor\b.*?(needed|required|should)',
                r'\bcleanup\b.*?(needed|required|should)',
                r'\brestructure\b.*?(needed|required|should)',
                r'\breorganize\b.*?(needed|required|should)',
                r'\bsimplify\b.*?(needed|required|should)',
                r'\boptimize\b.*?(needed|required|should)',
                r'\bimprove\b.*?(needed|required|should)',
                r'\benhance\b.*?(needed|required|should)',
                r'\bmodernize\b.*?(needed|required|should)',
                r'\bupdate\b.*?(needed|required|should)',
                r'\bupgrade\b.*?(needed|required|should)',
            ],
        }

        # Context exclusion patterns (legitimate technical terms)
        self.exclusion_patterns = [
            # Performance monitoring (legitimate)
            r'\bperformance\s+monitoring\b',
            r'\bperformance\s+analysis\b',
            r'\bperformance\s+benchmark\b',
            r'\befficient\s+implementation\b',
            
            # Simulation and testing (legitimate)
            r'\bsimulation\s+environment\b',
            r'\bsimulate\s+network\s+conditions\b',
            r'\bsimulation\s+mode\b',
            
            # Authentication (legitimate)
            r'\bbasic\s+authentication\b',
            r'\bbasic\s+configuration\b',
            r'\bsimple\s+interface\b',
            r'\bsimple\s+api\b',
            
            # Mock and testing (legitimate)
            r'\bmock\s+object\b',
            r'\bmock\s+service\b',
            r'\bmock\s+data\b',
            r'\bmock\s+response\b',
            
            # Documentation patterns (legitimate)
            r'\bcurrent\s+implementation\b.*?(uses|provides|supports)',
            r'\bthis\s+implementation\b.*?(uses|provides|supports)',
            r'\bthe\s+implementation\b.*?(uses|provides|supports)',
            
            # Architecture (legitimate)
            r'\barchitecture\s+note\b',
            r'\bdesign\s+note\b',
            r'\bpattern\s+note\b',
            r'\bdependency\s+injection\b',
            r'\bresource\s+management\b',
            
            # Logging (legitimate)
            r'console\.(log|warn|error|info)',
            r'\blogging\s+implementation\b',
            r'\berror\s+handling\b',
        ]

        # Documentation indicators
        self.doc_indicators = [
            r'@param', r'@return', r'@throws', r'@author', r'@date',
            r'@version', r'@description', r'@example', r'@see', r'@since',
            r'@deprecated', r'\*\s*\*\s*\*', r'^\s*/\*\*', r'^\s*///',
            r'^\s*# ', r'^\s*## ', r'^\s*### ',
        ]

        # TODO indicators
        self.todo_indicators = [
            r'\bneed\s+to\b', r'\bshould\s+be\b', r'\bmust\s+be\b',
            r'\bhas\s+to\b', r'\brequired\s+to\b', r'\bmissing\b',
            r'\bincomplete\b', r'\bpartial\b', r'\bunfinished\b',
            r'\bwork\s+in\s+progress\b', r'\bwip\b',
        ]

        self.results = []
        self.stats = defaultdict(int)

    def should_ignore_file(self, file_path: Path) -> bool:
        """Check if file should be ignored."""
        path_str = str(file_path)
        return any(re.search(pattern, path_str, re.IGNORECASE) 
                  for pattern in self.ignored_patterns)

    def detect_language(self, file_path: Path) -> Optional[str]:
        """Detect programming language from file extension."""
        suffix = file_path.suffix.lower()
        
        # Handle special cases
        if file_path.name.lower() in ['dockerfile', 'dockerfile.dev']:
            return 'dockerfile'
        
        for language, config in self.language_patterns.items():
            if suffix in config['extensions'] or file_path.name.lower() in config['extensions']:
                return language
        
        return None

    def extract_comments_from_file(self, file_path: Path) -> List[Tuple[int, str, str]]:
        """Extract all comments from file. Returns (line_num, comment, comment_type)."""
        language = self.detect_language(file_path)
        if not language:
            return []

        config = self.language_patterns[language]
        comments = []

        try:
            with open(file_path, 'r', encoding='utf-8', errors='ignore') as f:
                lines = f.readlines()

            in_multiline = False
            multiline_content = []
            multiline_start_line = 0

            for line_num, line in enumerate(lines, 1):
                original_line = line
                line = line.strip()

                if not line:
                    continue

                # Handle multi-line comments
                if config['multi_line_start'] and config['multi_line_end']:
                    if re.search(config['multi_line_start'], line):
                        in_multiline = True
                        multiline_content = []
                        multiline_start_line = line_num
                        continue

                    if in_multiline:
                        if re.search(config['multi_line_end'], line):
                            in_multiline = False
                            if multiline_content:
                                comment = ' '.join(multiline_content).strip()
                                if comment:
                                    comments.append((multiline_start_line, comment, 'multiline'))
                            multiline_content = []
                        else:
                            multiline_content.append(line)
                        continue

                # Extract single-line comments
                if config['single_line'] and re.search(config['single_line'], line):
                    # Determine comment type
                    comment_type = 'single'
                    if config['doc_comment'] and re.search(config['doc_comment'], line):
                        comment_type = 'doc'
                    
                    # Remove comment prefix
                    if language in ['rust', 'javascript', 'typescript', 'go', 'java', 'csharp', 'cpp', 'c', 'php']:
                        comment = re.sub(r'^\s*//\s*', '', line)
                        comment = re.sub(r'^\s*///\s*', '', comment)
                    elif language in ['python', 'ruby', 'shell', 'yaml', 'toml', 'dockerfile']:
                        comment = re.sub(r'^\s*#\s*', '', line)
                    elif language == 'markdown':
                        comment = re.sub(r'^\s*<!--\s*', '', line)
                        comment = re.sub(r'\s*-->$', '', comment)
                    elif language == 'sql':
                        comment = re.sub(r'^\s*--\s*', '', line)

                    if comment:
                        comments.append((line_num, comment, comment_type))

        except Exception as e:
            print(f"⚠️ Error reading {file_path}: {e}")

        return comments

    def is_excluded_pattern(self, comment: str) -> bool:
        """Check if comment matches exclusion patterns."""
        return any(re.search(pattern, comment, re.IGNORECASE) 
                  for pattern in self.exclusion_patterns)

    def is_documentation_comment(self, comment: str) -> bool:
        """Check if comment appears to be documentation."""
        return any(re.search(indicator, comment, re.IGNORECASE) 
                  for indicator in self.doc_indicators)

    def has_todo_indicators(self, comment: str) -> bool:
        """Check if comment contains TODO indicators."""
        return any(re.search(indicator, comment, re.IGNORECASE) 
                  for indicator in self.todo_indicators)

    def calculate_context_score(self, comment: str, line_num: int, file_path: Path) -> float:
        """Calculate context score to determine if this is a real TODO."""
        score = 0.0
        
        # Documentation indicators (reduce score)
        if self.is_documentation_comment(comment):
            score -= 0.5
        
        # TODO indicators (increase score)
        if self.has_todo_indicators(comment):
            score += 0.3
        
        # Generated file (reduce score)
        if self.is_generated_file(file_path):
            score -= 0.4
        
        # Comment length (very short likely not TODO)
        if len(comment.strip()) < 15:
            score -= 0.2
        
        # Documentation starters (reduce score)
        doc_starters = ['note:', 'current', 'this', 'the', 'implementation', 'method', 'function', 'class', 'interface']
        if any(comment.lower().startswith(starter) for starter in doc_starters):
            score -= 0.2
        
        # Action words (increase score)
        action_words = ['need', 'should', 'must', 'fix', 'implement', 'complete', 'add', 'remove', 'replace']
        if any(word in comment.lower() for word in action_words):
            score += 0.2
        
        return max(-1.0, min(1.0, score))

    def is_generated_file(self, file_path: Path) -> bool:
        """Check if file appears to be generated."""
        path_str = str(file_path)
        generated_indicators = [
            r'\.next\b', r'generated', r'build/', r'dist/', r'target/',
            r'node_modules', r'\.min\.', r'\.bundle\.', r'\.chunk\.',
        ]
        return any(re.search(indicator, path_str, re.IGNORECASE) 
                  for indicator in generated_indicators)

    def analyze_comment(self, comment: str, line_num: int, file_path: Path, language: str) -> Optional[CommentAnalysis]:
        """Analyze a single comment for TODO patterns."""
        
        # Skip if excluded pattern
        if self.is_excluded_pattern(comment):
            return None

        # Calculate context score
        context_score = self.calculate_context_score(comment, line_num, file_path)
        
        # Skip if strongly suggests documentation
        if context_score < -0.3:
            return None

        matches = defaultdict(list)
        confidence_scores = []
        
        # Check explicit patterns (highest confidence)
        for pattern in self.explicit_patterns['explicit_todos']:
            if re.search(pattern, comment, re.IGNORECASE):
                matches['explicit_todos'].append(pattern)
                base_confidence = 1.0
                adjusted_confidence = max(0.1, base_confidence + context_score * 0.3)
                confidence_scores.append(('explicit', adjusted_confidence))
                self.stats[pattern] += 1

        # Check hidden patterns
        for category, patterns in self.hidden_patterns.items():
            for pattern in patterns:
                if re.search(pattern, comment, re.IGNORECASE):
                    matches[category].append(pattern)
                    base_confidence = 0.8
                    adjusted_confidence = max(0.1, base_confidence + context_score * 0.2)
                    confidence_scores.append((category, adjusted_confidence))
                    self.stats[pattern] += 1

        if not matches:
            return None

        # Calculate overall confidence
        overall_confidence = max([score for _, score in confidence_scores]) if confidence_scores else 0.0
        
        # Determine TODO type flags
        is_explicit = 'explicit_todos' in matches
        is_hidden = not is_explicit and any(cat != 'explicit_todos' for cat in matches)
        is_placeholder = any(cat in matches for cat in ['placeholder_code', 'hardcoded_values'])
        is_temporary = any(cat in matches for cat in ['temporary_solutions', 'future_improvements'])
        is_incomplete = any(cat in matches for cat in ['incomplete_implementation', 'error_handling', 'testing_debt'])
        is_technical_debt = any(cat in matches for cat in ['performance_issues', 'security_concerns', 'refactoring_debt', 'documentation_debt'])

        return CommentAnalysis(
            file_path=str(file_path.relative_to(self.root_dir)),
            line_number=line_num,
            language=language,
            comment_text=comment,
            confidence_score=overall_confidence,
            pattern_category=max(matches.keys(), key=lambda k: len(matches[k])),
            matched_patterns=[p for patterns in matches.values() for p in patterns],
            context_score=context_score,
            is_explicit_todo=is_explicit,
            is_hidden_todo=is_hidden,
            is_placeholder=is_placeholder,
            is_temporary=is_temporary,
            is_incomplete=is_incomplete,
            is_technical_debt=is_technical_debt,
        )

    def analyze_directory(self, languages: Optional[List[str]] = None, min_confidence: float = 0.5) -> Dict[str, Any]:
        """Analyze all files in directory for comprehensive TODO patterns."""
        print(f"🔍 Starting exhaustive comment analysis in: {self.root_dir}")
        print(f"📊 Minimum confidence threshold: {min_confidence}")
        print(f"⏰ Analysis started at: {time.strftime('%Y-%m-%d %H:%M:%S')}")
        print()

        # Get all files
        all_files = []
        for language, config in self.language_patterns.items():
            if languages and language not in languages:
                continue
            for ext in config['extensions']:
                if ext.startswith('.'):
                    all_files.extend(self.root_dir.rglob(f'*{ext}'))
                else:
                    # Handle special cases like Dockerfile
                    all_files.extend(self.root_dir.rglob(ext))

        # Filter ignored files
        non_ignored_files = [f for f in all_files if not self.should_ignore_file(f)]

        print(f"📁 Found {len(all_files)} total files")
        print(f"📁 Found {len(non_ignored_files)} non-ignored files")

        # Language counts
        language_counts = defaultdict(int)
        for file_path in non_ignored_files:
            language = self.detect_language(file_path)
            if language:
                language_counts[language] += 1

        print("📊 Files by language:")
        for lang, count in sorted(language_counts.items()):
            print(f"  {lang}: {count} files")
        print()

        # Analyze files
        file_results = {}
        total_comments = 0
        total_todos = 0
        
        for i, file_path in enumerate(non_ignored_files, 1):
            if i % 50 == 0:
                print(f"⏳ Progress: {i}/{len(non_ignored_files)} files analyzed...")
            
            language = self.detect_language(file_path)
            if not language:
                continue

            comments = self.extract_comments_from_file(file_path)
            total_comments += len(comments)
            
            file_todos = []
            for line_num, comment, comment_type in comments:
                analysis = self.analyze_comment(comment, line_num, file_path, language)
                if analysis and analysis.confidence_score >= min_confidence:
                    file_todos.append(analysis)
                    total_todos += 1

            if file_todos:
                file_results[str(file_path.relative_to(self.root_dir))] = {
                    'language': language,
                    'total_comments': len(comments),
                    'todos': [asdict(todo) for todo in file_todos]
                }

        # Calculate statistics
        analysis_time = time.time() - self.start_time
        
        # Group by pattern categories
        pattern_groups = defaultdict(list)
        for analysis in self.results:
            pattern_groups[analysis.pattern_category].append(analysis)

        # Summary statistics
        summary = {
            'analysis_metadata': {
                'start_time': time.strftime('%Y-%m-%d %H:%M:%S', time.localtime(self.start_time)),
                'duration_seconds': round(analysis_time, 2),
                'root_directory': str(self.root_dir),
                'min_confidence_threshold': min_confidence,
            },
            'file_statistics': {
                'total_files_found': len(all_files),
                'non_ignored_files': len(non_ignored_files),
                'ignored_files': len(all_files) - len(non_ignored_files),
                'files_with_todos': len(file_results),
                'language_breakdown': dict(language_counts),
            },
            'comment_statistics': {
                'total_comments_analyzed': total_comments,
                'total_todos_found': total_todos,
                'todos_by_confidence': {
                    'high_confidence_90_plus': sum(1 for r in self.results if r.confidence_score >= 0.9),
                    'medium_confidence_70_89': sum(1 for r in self.results if 0.7 <= r.confidence_score < 0.9),
                    'low_confidence_50_69': sum(1 for r in self.results if 0.5 <= r.confidence_score < 0.7),
                },
                'todos_by_type': {
                    'explicit_todos': sum(1 for r in self.results if r.is_explicit_todo),
                    'hidden_todos': sum(1 for r in self.results if r.is_hidden_todo),
                    'placeholder_code': sum(1 for r in self.results if r.is_placeholder),
                    'temporary_solutions': sum(1 for r in self.results if r.is_temporary),
                    'incomplete_implementations': sum(1 for r in self.results if r.is_incomplete),
                    'technical_debt': sum(1 for r in self.results if r.is_technical_debt),
                },
                'pattern_statistics': dict(self.stats),
            },
            'files': file_results,
        }

        print(f"✅ Analysis complete in {analysis_time:.2f} seconds")
        print(f"📊 Total comments analyzed: {total_comments}")
        print(f"🎯 Total TODOs found: {total_todos}")
        print(f"📁 Files with TODOs: {len(file_results)}")

        return summary


def main():
    """Main function to run exhaustive comment analysis."""
    import argparse

    parser = argparse.ArgumentParser(
        description='Exhaustive comment analysis for hidden TODO patterns'
    )
    parser.add_argument('--root', default='.',
                        help='Root directory to analyze (default: current directory)')
    parser.add_argument('--languages', nargs='+',
                        help='Specific languages to analyze (e.g., rust python javascript)')
    parser.add_argument('--output-json', help='Output JSON file for detailed results')
    parser.add_argument('--output-md', help='Output Markdown report file')
    parser.add_argument('--min-confidence', type=float, default=0.5,
                        help='Minimum confidence threshold (0.0-1.0, default: 0.5)')
    parser.add_argument('--verbose', '-v', action='store_true',
                        help='Verbose output')

    args = parser.parse_args()

    analyzer = ExhaustiveCommentAnalyzer(args.root)
    results = analyzer.analyze_directory(args.languages, args.min_confidence)

    # Save results
    if args.output_json:
        with open(args.output_json, 'w') as f:
            json.dump(results, f, indent=2)
        print(f"💾 Detailed results saved to: {args.output_json}")

    if args.output_md:
        # Generate markdown report
        report = generate_markdown_report(results)
        with open(args.output_md, 'w') as f:
            f.write(report)
        print(f"📄 Report saved to: {args.output_md}")

    return results


def generate_markdown_report(results: Dict[str, Any]) -> str:
    """Generate comprehensive markdown report."""
    report = []
    report.append("# Exhaustive Comment Analysis Report")
    report.append("=" * 60)
    report.append("")
    
    # Metadata
    meta = results['analysis_metadata']
    report.append("## Analysis Metadata")
    report.append(f"- **Start Time**: {meta['start_time']}")
    report.append(f"- **Duration**: {meta['duration_seconds']} seconds")
    report.append(f"- **Root Directory**: {meta['root_directory']}")
    report.append(f"- **Confidence Threshold**: {meta['min_confidence_threshold']}")
    report.append("")
    
    # File statistics
    file_stats = results['file_statistics']
    report.append("## File Statistics")
    report.append(f"- **Total Files Found**: {file_stats['total_files_found']}")
    report.append(f"- **Non-ignored Files**: {file_stats['non_ignored_files']}")
    report.append(f"- **Ignored Files**: {file_stats['ignored_files']}")
    report.append(f"- **Files with TODOs**: {file_stats['files_with_todos']}")
    report.append("")
    
    # Language breakdown
    report.append("### Files by Language")
    for lang, count in sorted(file_stats['language_breakdown'].items()):
        report.append(f"- **{lang}**: {count} files")
    report.append("")
    
    # Comment statistics
    comment_stats = results['comment_statistics']
    report.append("## Comment Statistics")
    report.append(f"- **Total Comments Analyzed**: {comment_stats['total_comments_analyzed']}")
    report.append(f"- **Total TODOs Found**: {comment_stats['total_todos_found']}")
    report.append("")
    
    # Confidence breakdown
    confidence_stats = comment_stats['todos_by_confidence']
    report.append("### TODOs by Confidence Level")
    report.append(f"- **High Confidence (≥0.9)**: {confidence_stats['high_confidence_90_plus']}")
    report.append(f"- **Medium Confidence (0.7-0.89)**: {confidence_stats['medium_confidence_70_89']}")
    report.append(f"- **Low Confidence (0.5-0.69)**: {confidence_stats['low_confidence_50_69']}")
    report.append("")
    
    # Type breakdown
    type_stats = comment_stats['todos_by_type']
    report.append("### TODOs by Type")
    report.append(f"- **Explicit TODOs**: {type_stats['explicit_todos']}")
    report.append(f"- **Hidden TODOs**: {type_stats['hidden_todos']}")
    report.append(f"- **Placeholder Code**: {type_stats['placeholder_code']}")
    report.append(f"- **Temporary Solutions**: {type_stats['temporary_solutions']}")
    report.append(f"- **Incomplete Implementations**: {type_stats['incomplete_implementations']}")
    report.append(f"- **Technical Debt**: {type_stats['technical_debt']}")
    report.append("")
    
    # Top patterns
    if comment_stats['pattern_statistics']:
        report.append("### Most Common Patterns")
        sorted_patterns = sorted(comment_stats['pattern_statistics'].items(), 
                               key=lambda x: x[1], reverse=True)
        for pattern, count in sorted_patterns[:20]:
            if count > 0:
                report.append(f"- `{pattern}`: {count} occurrences")
        report.append("")
    
    # Files with most TODOs
    if results['files']:
        report.append("## Files with TODOs")
        file_todo_counts = []
        for file_path, data in results['files'].items():
            todo_count = len(data['todos'])
            file_todo_counts.append((file_path, data['language'], todo_count))
        
        file_todo_counts.sort(key=lambda x: x[2], reverse=True)
        for file_path, language, count in file_todo_counts[:20]:
            report.append(f"- `{file_path}` ({language}): {count} TODOs")
        
        if len(file_todo_counts) > 20:
            report.append(f"- ... and {len(file_todo_counts) - 20} more files")
        report.append("")
    
    return "\n".join(report)


if __name__ == '__main__':
    main()
