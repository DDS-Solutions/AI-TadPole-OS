#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Infrastructure:Execution

### AI Assist Note
**Python Execution Utilities (py_utils)**: Shared module containing helper routines 
for Windows UTF-8 console output re-buffering, color logging, and standardized 
CLI reporting outputs.

### 🔍 Debugging & Observability
- **Trace Scope**: `execution::py_utils`
"""

import sys
import io

# ANSI colors
class Colors:
    HEADER = '\033[95m'
    BLUE = '\033[94m'
    CYAN = '\033[96m'
    GREEN = '\033[92m'
    YELLOW = '\033[93m'
    RED = '\033[91m'
    ENDC = '\033[0m'
    BOLD = '\033[1m'

def init_utf8():
    """Configures stdout and stderr to handle UTF-8 output on Windows console hosts."""
    if sys.platform == "win32":
        try:
            if hasattr(sys.stdout, 'buffer') and getattr(sys.stdout, 'encoding', '').lower() != 'utf-8':
                sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8', errors='replace')
            if hasattr(sys.stderr, 'buffer') and getattr(sys.stderr, 'encoding', '').lower() != 'utf-8':
                sys.stderr = io.TextIOWrapper(sys.stderr.buffer, encoding='utf-8', errors='replace')
        except Exception:
            pass

def print_ok(text: str):
    """Print standard green success message."""
    print(f"{Colors.GREEN}[OK] {text}{Colors.ENDC}")

def print_warn(text: str):
    """Print standard yellow warning message."""
    print(f"{Colors.YELLOW}[WARN] {text}{Colors.ENDC}")

def print_err(text: str):
    """Print standard red error message."""
    print(f"{Colors.RED}[FAIL] {text}{Colors.ENDC}")

def print_step(text: str):
    """Print standard blue step progress message."""
    print(f"{Colors.BOLD}{Colors.BLUE}[STEP] {text}{Colors.ENDC}")

def print_header(text: str):
    """Print cyan border around a centered title block."""
    print(f"\n{Colors.BOLD}{Colors.CYAN}{'='*70}{Colors.ENDC}")
    print(f"{Colors.BOLD}{Colors.CYAN}{text.center(70)}{Colors.ENDC}")
    print(f"{Colors.BOLD}{Colors.CYAN}{'='*70}{Colors.ENDC}\n")
