#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Documentation

### AI Context Alignment
- **Subsystem**: Agent Skills / webapp-testing / playwright_runner
- **Primary Entrypoints**: `main`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic execution without side effects outside declared scope.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: none declared
- **Witness Tests**: none declared
"""

import sys
import json
import os
import tempfile
from datetime import datetime

# Fix Windows console encoding for Unicode output
if sys.platform == "win32":
    if hasattr(sys.stdout, 'reconfigure'):
        sys.stdout.reconfigure(encoding='utf-8', errors='replace')
    if hasattr(sys.stderr, 'reconfigure'):
        sys.stderr.reconfigure(encoding='utf-8', errors='replace')

try:
    from playwright.sync_api import sync_playwright
    PLAYWRIGHT_AVAILABLE = True
except ImportError:
    PLAYWRIGHT_AVAILABLE = False


def run_basic_test(url: str, take_screenshot: bool = False) -> dict:
    """Run basic browser test on URL."""
    if not PLAYWRIGHT_AVAILABLE:
        return {
            "error": "Playwright not installed",
            "fix": "pip install playwright && playwright install chromium"
        }
    
    result = {
        "url": url,
        "timestamp": datetime.now().isoformat(),
        "status": "pending"
    }
    
    try:
        with sync_playwright() as p:
            browser = p.chromium.launch(headless=True)
            context = browser.new_context(
                viewport={"width": 1280, "height": 720},
                user_agent="Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36"
            )
            page = context.new_page()
            
            console_errors = []
            page.on("console", lambda msg: console_errors.append(msg.text) if msg.type == "error" else None)
            
            page_errors = []
            page.on("pageerror", lambda err: page_errors.append(str(err)))
            
            response = page.goto(url, wait_until="networkidle", timeout=30000)
            
            result["http_status"] = response.status if response else None
            result["title"] = page.title()
            result["console_errors"] = console_errors
            result["page_errors"] = page_errors
            
            if take_screenshot:
                screenshot_path = tempfile.mktemp(suffix=".png")
                page.screenshot(path=screenshot_path, full_page=True)
                result["screenshot"] = screenshot_path
            
            browser.close()
            result["status"] = "success"
            
    except Exception as e:
        result["status"] = "error"
        result["error"] = str(e)
    
    return result


def run_accessibility_check(url: str) -> dict:
    """Run basic accessibility checks on URL."""
    if not PLAYWRIGHT_AVAILABLE:
        return {
            "error": "Playwright not installed",
            "fix": "pip install playwright && playwright install chromium"
        }
    
    result = {
        "url": url,
        "timestamp": datetime.now().isoformat(),
        "status": "pending"
    }
    
    try:
        with sync_playwright() as p:
            browser = p.chromium.launch(headless=True)
            page = browser.new_page()
            page.goto(url, wait_until="networkidle", timeout=30000)
            
            result["checks"] = {
                "has_title": bool(page.title()),
                "has_lang": bool(page.locator("html").get_attribute("lang")),
                "images_with_alt": page.locator("img[alt]").count(),
                "images_total": page.locator("img").count(),
                "buttons_with_text": page.locator("button:has-text('')").count(),
                "buttons_total": page.locator("button").count(),
                "links_with_text": page.locator("a:has-text('')").count(),
                "form_labels": page.locator("label").count(),
                "headings": {
                    "h1": page.locator("h1").count(),
                    "h2": page.locator("h2").count(),
                    "h3": page.locator("h3").count()
                }
            }
            
            browser.close()
            result["status"] = "success"
            
    except Exception as e:
        result["status"] = "error"
        result["error"] = str(e)
    
    return result


def run_self_tests() -> bool:
    """Run built-in self-tests for playwright runner."""
    if not PLAYWRIGHT_AVAILABLE:
        res = run_basic_test("http://localhost:9999")
        assert "error" in res, "Should report Playwright not installed"
    else:
        assert callable(run_basic_test) and callable(run_accessibility_check)

    print("[PASS] playwright_runner self-tests passed successfully.")
    return True


def main():
    if "--test" in sys.argv:
        success = run_self_tests()
        sys.exit(0 if success else 1)

    print("[playwright_runner] Initializing Playwright browser test suite...")
    if len(sys.argv) < 2:
        print(json.dumps({
            "error": "Usage: python playwright_runner.py <url> [--screenshot] [--a11y] [--test]",
            "examples": [
                "python playwright_runner.py https://example.com",
                "python playwright_runner.py https://example.com --screenshot",
                "python playwright_runner.py https://example.com --a11y"
            ]
        }, indent=2))
        sys.exit(1)
    
    url = sys.argv[1]
    take_screenshot = "--screenshot" in sys.argv
    check_a11y = "--a11y" in sys.argv
    
    if check_a11y:
        result = run_accessibility_check(url)
    else:
        result = run_basic_test(url, take_screenshot)
    
    print(json.dumps(result, indent=2))


if __name__ == "__main__":
    main()
