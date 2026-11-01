"""@docs ARCHITECTURE:Infrastructure:Execution
### AI Assist Note
Verify that reported completion alone cannot pass a mission benchmark.
### 🔍 Debugging & Observability
Run with python -m unittest discover -s tests -p test_mission_benchmark.py.
"""
import importlib.util
from pathlib import Path
import tempfile
import unittest

spec = importlib.util.spec_from_file_location("mission_benchmark", Path(__file__).resolve().parents[1] / "execution/mission_benchmark.py")
benchmark = importlib.util.module_from_spec(spec)
spec.loader.exec_module(benchmark)


class MissionBenchmarkTests(unittest.TestCase):
    def setUp(self):
        self.folder = tempfile.TemporaryDirectory()
        self.addCleanup(self.folder.cleanup)
        self.root = Path(self.folder.name)
        (self.root / "source.rs").write_text("source", encoding="utf-8")
        self.case = {"case_id":"blind-1", "purpose":"inward", "brief":"Inspect the source", "agents":[{"id":"Alpha","model":"local-mock","execution_temperature":0.2}], "sources":["source.rs"], "rubric":{"root_cause":4,"regression":2}, "history_policy":"fresh"}
        self.frozen = benchmark.freeze(self.case, self.root)
        self.run = {"case_fingerprint":self.frozen["case_fingerprint"], "execution_checks":dict.fromkeys(["handoffs_once","reports_delivered","scope_preserved","budget_preserved"],True)}

    def test_completed_execution_without_review_is_unscored(self):
        self.run["status"] = "completed"
        result = benchmark.score(self.frozen,self.run,root=self.root)
        self.assertEqual(result["quality_status"],"UNSCORED")
        self.assertIsNone(result["score"])

    def test_grade_requires_evidence_and_execution_success(self):
        grade = {"case_fingerprint":self.frozen["case_fingerprint"], "reviewer_type":"independent", "points":{"root_cause":4,"regression":2}, "evidence_files":["source.rs"]}
        self.assertEqual(benchmark.score(self.frozen,self.run,grade,self.root)["quality_status"],"PASS")
        self.run["execution_checks"]["handoffs_once"] = False
        self.assertEqual(benchmark.score(self.frozen,self.run,grade,self.root)["quality_status"],"FAIL")
        grade["evidence_files"] = []
        with self.assertRaises(ValueError): benchmark.score(self.frozen,self.run,grade,self.root)

    def test_changed_source_or_temperature_is_not_comparable(self):
        first = benchmark.score(self.frozen,self.run,root=self.root)
        self.case["agents"][0]["execution_temperature"] = 0.9
        different = benchmark.freeze(self.case,self.root)
        with self.assertRaises(ValueError): benchmark.compare(first,{"case_fingerprint":different["case_fingerprint"]})
        (self.root / "source.rs").write_text("changed",encoding="utf-8")
        self.assertNotEqual(different["case_fingerprint"],benchmark.freeze(self.case,self.root)["case_fingerprint"])

    def test_tampering_and_out_of_range_grades_are_rejected(self):
        grade = {"case_fingerprint":self.frozen["case_fingerprint"], "reviewer_type":"independent", "points":{"root_cause":5,"regression":2}, "evidence_files":["source.rs"]}
        with self.assertRaises(ValueError): benchmark.score(self.frozen,self.run,grade,self.root)
        self.frozen["conditions"]["brief"] = "changed after freeze"
        with self.assertRaises(ValueError): benchmark.score(self.frozen,self.run,root=self.root)


if __name__ == "__main__": unittest.main()
