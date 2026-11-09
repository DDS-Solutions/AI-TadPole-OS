"""@docs ARCHITECTURE:Core

### AI Assist Note
Exercise verifier failure modes without contacting the engine or any provider.
### 🔍 Debugging & Observability
Missing checks and malformed evidence cannot yield a passing verification report.
"""
import importlib.util
import unittest
from pathlib import Path

SPEC = importlib.util.spec_from_file_location("verify_mock_smb", Path(__file__).parents[1] / "execution/verify_mock_smb.py")
verifier = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(verifier)


class VerifierTests(unittest.TestCase):
    def test_restricts_target_to_loopback_origins(self):
        for target in ("https://example.com", "http://localhost.evil", "http://user:secret@localhost:8000", "http://localhost:8000/v1", "http://localhost:8000/?x=1", "file:///tmp/x"):
            with self.subTest(target=target), self.assertRaises(ValueError):
                verifier.local_target(target)
        self.assertEqual(verifier.local_target("http://localhost:8000/"), "http://localhost:8000")
        self.assertEqual(verifier.local_target("http://[::1]:8000"), "http://[::1]:8000")

    def test_missing_or_failed_check_cannot_pass(self):
        self.assertFalse(verifier.finalized("Mock-1-Product", {}, {})["passed"])
        complete = dict.fromkeys(verifier.REQUIRED, True)
        self.assertTrue(verifier.finalized("Mock-1-Product", complete, {})["passed"])
        complete["safe_rejection"] = False
        self.assertFalse(verifier.finalized("Mock-1-Product", complete, {})["passed"])
        self.assertFalse(verifier.finalized("Mock-1-Product", dict.fromkeys(verifier.REQUIRED, "true"), {})["passed"])
        self.assertFalse(verifier.finalized("Mock-1-Product", dict.fromkeys(verifier.REQUIRED, True), {}, "interrupted")["passed"])

    def test_malformed_server_response_is_recorded_as_incomplete(self):
        class BrokenClient:
            def call(self, *args):
                return 200, {"verification": {"verified": True}}
        report = verifier.verify_company(BrokenClient(), "Mock-1-Product", "attempt")
        self.assertFalse(report["passed"])
        self.assertEqual(len(report["checks"]), len(verifier.REQUIRED))
        self.assertTrue(report["error"])

    def test_transport_error_does_not_reveal_credentials(self):
        class BrokenClient:
            def call(self, *args):
                raise RuntimeError("secret credential or raw remote response")
        report = verifier.verify_company(BrokenClient(), "Mock-2-Service", "attempt")
        self.assertNotIn("secret", report["error"])
        self.assertFalse(report["passed"])

    def test_redirects_are_not_followed(self):
        self.assertIsNone(verifier.NoRedirects().redirect_request(None, None, 302, "redirect", {}, "https://example.com"))

    def test_malformed_run_ids_cannot_select_another_endpoint(self):
        for identity in ("../../engine/kill", "abc?x=1", "abc/decision", None):
            with self.subTest(identity=identity), self.assertRaises((ValueError, TypeError)):
                verifier.run_path("/v1/business/companies/Mock-1-Product", {"run_id": identity})


if __name__ == "__main__":
    unittest.main()
