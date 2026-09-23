"""
@docs ARCHITECTURE:Documentation

### AI Assist Note
- **Subsystem**: API Reference Generator Regression Tests

### 🔍 Debugging & Observability
- Tests use source fixtures and read-only repository scans.
"""

import re
import unittest
from execution.generate_api_reference import ROOT, extract_route_docs


class ApiReferenceTests(unittest.TestCase):
    def test_attributes_and_description_after_slug_are_preserved(self):
        source = '''/// POST /v1/agents/:id/reset
/// @docs API_REFERENCE:ResetAgent
/// Resets the agent.
#[tracing::instrument(
    skip(state),
    name = "reset"
)]
#[cfg(feature = "reset")]
pub async fn reset_agent() {}

/// GET /internal
/// No API reference annotation.
pub async fn internal() {}
'''
        self.assertEqual(extract_route_docs(source, 'agent.rs'), [{
            'method': 'POST', 'path': '/v1/agents/:id/reset',
            'description': 'Resets the agent.', 'slug': 'ResetAgent',
            'handler': 'pub async fn reset_agent', 'file': 'agent.rs',
        }])

    def test_all_annotated_routes_have_reference_entries(self):
        routes = ROOT / 'server-rs' / 'src' / 'routes'
        expected = set()
        actual = set()
        for path in routes.rglob('*.rs'):
            content = path.read_text(encoding='utf-8')
            expected.update(re.findall(r'@docs API_REFERENCE:(\w+)', content))
            actual.update(entry['slug'] for entry in extract_route_docs(content, path.name))
        self.assertIn('GetPendingOversight', expected)
        self.assertEqual(actual, expected)


if __name__ == '__main__':
    unittest.main()
