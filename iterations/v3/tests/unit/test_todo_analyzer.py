import sys
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
if str(ROOT) not in sys.path:
    sys.path.insert(0, str(ROOT))

from scripts.todo_analyzer import HiddenTodoAnalyzer  # noqa: E402


class HiddenTodoAnalyzerTests(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp_root = Path(__file__).resolve().parent
        self.analyzer = HiddenTodoAnalyzer(root_dir=str(self.tmp_root))

    def test_documentation_comments_are_excluded(self):
        comment = "@param user_data The serialized payload."
        self.assertTrue(self.analyzer.is_documentation_comment(comment))
        self.assertEqual(
            self.analyzer.analyze_comment(comment, 12, Path("src/service.py")),
            {},
        )

    def test_todo_indicators_raise_context_score(self):
        comment = "Need to replace this placeholder with the real implementation."
        score = self.analyzer.calculate_context_score(
            comment, 20, Path("src/core/module.py")
        )
        self.assertGreaterEqual(score, 0.29)

    def test_generated_files_reduce_context_score(self):
        comment = "Need to replace this placeholder with the real implementation."
        score = self.analyzer.calculate_context_score(
            comment, 5, Path("dist/generated.js")
        )
        self.assertLessEqual(score, -0.09)

    def test_explicit_todos_return_context_and_confidence(self):
        comment = "TODO: finalize the persistence layer wiring."
        result = self.analyzer.analyze_comment(
            comment, 32, Path("src/persistence/adapter.py")
        )

        self.assertIn("explicit_todos", result["matches"])
        self.assertTrue(result["matches"]["explicit_todos"])
        self.assertGreater(result["confidence_score"], 0.0)
        self.assertLessEqual(result["confidence_score"], 1.0)
        self.assertIn("context_score", result)


if __name__ == "__main__":
    unittest.main()
