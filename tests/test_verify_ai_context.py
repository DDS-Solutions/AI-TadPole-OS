"""
@docs ARCHITECTURE:Infrastructure:Execution

### AI Context Alignment
- **Subsystem**: Test Verification Suite / test_verify_ai_context
- **Primary Entrypoints**: `PythonAIContextVerifierTests`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` All tests must execute deterministically without network access.
  - enforced_by: `test_verify_ai_context_self`
- `[Behavioral]` Failure to provide an enforced_by witness for behavioral invariants must fail verification.
  - enforced_by: `test_behavioral_witness_enforcement`

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: `[test_verify_ai_context]`, `[my_telemetry_tag]`, `[rust_tag]`
- **Witness Tests**: `test_invariant_taxonomy_enforcement`, `test_behavioral_witness_enforcement`, `test_symbol_definition_resolution`, `test_telemetry_emitter_verification`, `test_verify_ai_context_self`
"""

import sys
import json
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
if str(ROOT) not in sys.path:
    sys.path.insert(0, str(ROOT))

from execution.verify_ai_context import (
    extract_metadata,
    verify_symbol_defined_in_file,
    tag_is_emitted,
    get_workspace_test_symbols,
    verify_file,
)


class PythonAIContextVerifierTests(unittest.TestCase):
    def setUp(self):
        print("[test_verify_ai_context] Setting up test case")

    def test_invariant_taxonomy_enforcement(self):
        """Validates that invariants without [Structural], [Behavioral], or [Advisory: UNVERIFIED] are rejected."""
        bad_sample = '''
"""
@docs ARCHITECTURE:Core

### AI Context Alignment
- **Subsystem**: Test

### ⚠️ Invariants & Non-Negotiables
- Invalid untagged invariant bullet here.

### 🔍 Debugging & Observability
- **Witness Tests**: none
"""
'''
        meta = extract_metadata(bad_sample)
        self.assertEqual(len(meta["invariants"]), 1)
        self.assertIsNone(meta["invariants"][0]["tag"])

        good_sample = '''
"""
@docs ARCHITECTURE:Core

### AI Context Alignment
- **Subsystem**: Test

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Valid structural invariant.
- `[Behavioral]` Valid behavioral invariant.
  - enforced_by: `test_sample_witness`
- `[Advisory: UNVERIFIED]` Valid advisory invariant.

### 🔍 Debugging & Observability
- **Witness Tests**: `test_sample_witness`
"""
'''
        meta_good = extract_metadata(good_sample)
        self.assertEqual(len(meta_good["invariants"]), 3)
        self.assertEqual(meta_good["invariants"][0]["tag"], "Structural")
        self.assertEqual(meta_good["invariants"][1]["tag"], "Behavioral")
        self.assertEqual(meta_good["invariants"][1]["witness"], "test_sample_witness")
        self.assertEqual(meta_good["invariants"][2]["tag"], "Advisory: UNVERIFIED")

    def test_behavioral_witness_enforcement(self):
        """Validates that [Behavioral] invariants require explicit enforced_by witness tests."""
        missing_enf = '''
"""
@docs ARCHITECTURE:Core

### AI Context Alignment
- **Subsystem**: Test

### ⚠️ Invariants & Non-Negotiables
- `[Behavioral]` Behavioral invariant without enforced_by clause.

### 🔍 Debugging & Observability
- **Witness Tests**: none
"""
'''
        meta = extract_metadata(missing_enf)
        self.assertEqual(meta["invariants"][0]["tag"], "Behavioral")
        self.assertIsNone(meta["invariants"][0]["witness"])

    def test_symbol_definition_resolution(self):
        """Validates AST and keyword definition heuristic resolution, and rejects comment mentions."""
        py_code = '''
def real_function():
    pass

class RealClass:
    pass

# def fake_in_comment():
#     pass
'''
        self.assertTrue(verify_symbol_defined_in_file("real_function", py_code, ".py"))
        self.assertTrue(verify_symbol_defined_in_file("RealClass", py_code, ".py"))
        self.assertFalse(verify_symbol_defined_in_file("fake_in_comment", py_code, ".py"))
        self.assertFalse(verify_symbol_defined_in_file("nonexistent_symbol", py_code, ".py"))

        # Rust comment stripping and definition resolution
        rs_code = '''
// pub fn fake_rust_comment() {}
/*
fn another_fake_in_block() {}
*/
pub async fn real_rust_fn() {}
pub struct RealRustStruct;
pub enum RealRustEnum {}
'''
        self.assertTrue(verify_symbol_defined_in_file("real_rust_fn", rs_code, ".rs"))
        self.assertTrue(verify_symbol_defined_in_file("RealRustStruct", rs_code, ".rs"))
        self.assertTrue(verify_symbol_defined_in_file("RealRustEnum", rs_code, ".rs"))
        self.assertFalse(verify_symbol_defined_in_file("fake_rust_comment", rs_code, ".rs"))
        self.assertFalse(verify_symbol_defined_in_file("another_fake_in_block", rs_code, ".rs"))

        # TS/JS definition resolution
        ts_code = '''
// const fakeTsComment = 1;
export const realArrowFn = () => {};
export function realTsFunction() {}
export interface RealInterface {}
export type RealType = string;
'''
        self.assertTrue(verify_symbol_defined_in_file("realArrowFn", ts_code, ".ts"))
        self.assertTrue(verify_symbol_defined_in_file("realTsFunction", ts_code, ".ts"))
        self.assertTrue(verify_symbol_defined_in_file("RealInterface", ts_code, ".ts"))
        self.assertTrue(verify_symbol_defined_in_file("RealType", ts_code, ".ts"))
        self.assertFalse(verify_symbol_defined_in_file("fakeTsComment", ts_code, ".ts"))

    def test_telemetry_emitter_verification(self):
        """Validates that declared telemetry tags must be emitted with recognized log/print emitters in code."""
        code_with_emitter = '''
def process_data():
    print("[my_telemetry_tag] Processing record")
'''
        self.assertTrue(tag_is_emitted("my_telemetry_tag", code_with_emitter))
        self.assertTrue(tag_is_emitted("none", code_with_emitter))
        self.assertTrue(tag_is_emitted("none declared", code_with_emitter))

        code_without_emitter = '''
def process_data():
    # Just a comment mentioning [my_telemetry_tag]
    return "my_telemetry_tag"
'''
        self.assertFalse(tag_is_emitted("my_telemetry_tag", code_without_emitter))

        rust_with_tracing = '''
fn process() {
    tracing::info!("[rust_tag] Event triggered");
}
'''
        self.assertTrue(tag_is_emitted("rust_tag", rust_with_tracing))

    def test_docs_presence_and_resolution(self):
        """Validates @docs extraction and multi-segment resolution."""
        sample_doc = '''
"""
@docs ARCHITECTURE:Infrastructure:Execution

### AI Context Alignment
- **Subsystem**: Test
"""
'''
        meta = extract_metadata(sample_doc)
        self.assertTrue(meta["has_docs"])
        self.assertEqual(meta["docs_link"], "ARCHITECTURE:Infrastructure:Execution")

    def test_attribute_anchored_rust_harvesting(self):
        """Validates that non-test functions like latest_data or contested are not harvested as tests."""
        rust_sample = '''
pub fn latest_data() {}
fn contested_election() {}

#[test]
fn real_harvested_test() {}

#[tokio::test]
async fn real_async_harvested_test() {}
'''
        import re
        harvested = set()
        for m in re.finditer(
            r'#\[(?:[\w:]*::)?test(?:\([^)]*\))?\](?:\s*#\[[^\]]+\])*\s*(?:pub(?:\([^)]+\))?\s+)?(?:async\s+)?fn\s+([a-zA-Z0-9_]+)',
            rust_sample
        ):
            harvested.add(m.group(1))

        self.assertIn("real_harvested_test", harvested)
        self.assertIn("real_async_harvested_test", harvested)
        self.assertNotIn("latest_data", harvested)
        self.assertNotIn("contested_election", harvested)

    def test_tag_agnostic_enforced_by_cross_check(self):
        """Validates that [Structural] invariants with enforced_by must be listed in Witness Tests."""
        sample_code = '''
"""
@docs ARCHITECTURE:Core

### AI Context Alignment
- **Subsystem**: Test

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Structural invariant with witness.
  - enforced_by: `test_structural_witness`

### 🔍 Debugging & Observability
- **Witness Tests**: `test_unrelated_witness`
"""
'''
        meta = extract_metadata(sample_code)
        enforced = {inv["witness"] for inv in meta["invariants"] if inv.get("witness")}
        declared = set(meta["witness_tests"])
        self.assertIn("test_structural_witness", enforced)
        self.assertNotIn("test_structural_witness", declared)

    def test_missing_docs_link_fails(self):
        """Validates that missing @docs tag is flagged as a failure finding."""
        sample_code = '''
"""
### AI Context Alignment
- **Subsystem**: Test

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Valid invariant.

### 🔍 Debugging & Observability
- **Witness Tests**: none
"""
'''
        meta = extract_metadata(sample_code)
        self.assertFalse(meta["has_docs"])

    def test_strip_order_and_raw_strings(self):
        """Validates that string literals containing // or comments are not truncated prematurely, and raw strings work."""
        from execution.verify_ai_context import strip_comments_and_strings
        rs_code = '''
fn endpoint() {
    let url = "https://example.com/api";
    let raw = r##"Some // raw string with comments and "nested" quotes"##;
    // real comment
}
'''
        stripped = strip_comments_and_strings(rs_code, ".rs")
        self.assertNotIn("real comment", stripped)
        self.assertIn("endpoint", stripped)

    def test_baseline_mode_loading(self):
        """Validates that baseline loading parses failure tuples and filters accordingly."""
        import tempfile
        from execution.verify_ai_context import load_baseline
        with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False, encoding='utf-8') as tf:
            json.dump({
                "failures": [
                    {
                        "file": "test_file.rs",
                        "findings": ["Missing '@docs' documentation link"]
                    }
                ]
            }, tf)
            temp_path = Path(tf.name)

        try:
            baseline = load_baseline(temp_path)
            self.assertIn(("test_file.rs", "Missing '@docs' documentation link"), baseline)
        finally:
            if temp_path.exists():
                temp_path.unlink()

    def test_verify_ai_context_self(self):
        """Verifies that verify_ai_context.py cleanly passes its own verification gates."""
        verifier_path = ROOT / "execution" / "verify_ai_context.py"
        res = verify_file(verifier_path)
        print(f"[test_verify_ai_context] Self-verification result: {res}")
        self.assertTrue(res["passed"], f"Self-verification failed with findings: {res['findings']}")
        self.assertEqual(len(res["findings"]), 0)


if __name__ == "__main__":
    unittest.main()
