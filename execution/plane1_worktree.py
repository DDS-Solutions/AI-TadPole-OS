"""
@docs ARCHITECTURE:Core, ARCHITECTURE:Sandboxing

### AI Context Alignment
- **Subsystem**: Sovereign Engine / Plane 1 Sandboxing
- **Primary Entrypoints**: `main`, `create_worktree`, `test_worktree`, `merge_worktree`, `abort_worktree`, `list_worktrees`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic worktree sandboxing under .tmp/worktrees/ with zero root working tree pollution.
- `[Structural]` Safe rollback: abort unconditionally purges worktree and temp branch without leaving debris.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: `[Plane1Worktree]`
- **Witness Tests**: none declared
"""

import os
import sys
import shutil
import argparse
import subprocess
from pathlib import Path
from typing import List, Dict, Optional

PROJECT_ROOT = Path(__file__).resolve().parent.parent
WORKTREES_ROOT = PROJECT_ROOT / ".tmp" / "worktrees"

def run_git(args: List[str], cwd: Path = PROJECT_ROOT, check: bool = True) -> subprocess.CompletedProcess:
    """Run git command and return CompletedProcess."""
    return subprocess.run(
        ["git"] + args,
        cwd=str(cwd),
        capture_output=True,
        text=True,
        check=check,
    )

def validate_slug(task_slug: str) -> str:
    """Ensure task slug is safe for branch and directory names."""
    slug = task_slug.strip()
    if not slug or len(slug) > 64:
        raise ValueError("Task slug must be non-empty and <= 64 characters.")
    if not all(c.isalnum() or c in ("-", "_") for c in slug):
        raise ValueError(f"Task slug '{slug}' contains invalid characters. Use alphanumeric, hyphens, and underscores only.")
    return slug

def get_worktree_path(task_slug: str) -> Path:
    return WORKTREES_ROOT / task_slug

def get_branch_name(task_slug: str) -> str:
    return f"plane1/{task_slug}"

def create_worktree(task_slug: str, base_ref: str = "HEAD") -> int:
    """Create an isolated git worktree under .tmp/worktrees/<task-slug>."""
    slug = validate_slug(task_slug)
    wt_path = get_worktree_path(slug)
    branch = get_branch_name(slug)

    print(f"\n[Plane1Worktree] Initializing sandbox for task: '{slug}'")
    
    # Ensure parent worktrees dir exists
    WORKTREES_ROOT.mkdir(parents=True, exist_ok=True)

    # Check if worktree directory already exists
    if wt_path.exists():
        print(f"[Plane1Worktree] Worktree path '{wt_path}' already exists.")
        print(f"[Plane1Worktree] Cleaning up previous instance first...")
        abort_worktree(slug)

    # Check if branch exists and remove if stale
    try:
        proc = run_git(["branch", "--list", branch], check=False)
        if branch in proc.stdout:
            print(f"[Plane1Worktree] Pruning stale branch '{branch}'...")
            run_git(["branch", "-D", branch], check=False)
    except Exception:
        pass

    # Create the worktree on a new branch
    cmd = ["worktree", "add", "-b", branch, str(wt_path), base_ref]
    proc = run_git(cmd, check=False)
    if proc.returncode != 0:
        print(f"[Plane1Worktree] ERROR: Failed to create worktree:\n{proc.stderr}")
        return 1

    # Link/Copy .env if it exists in root so runtime configs are available in sandbox
    root_env = PROJECT_ROOT / ".env"
    wt_env = wt_path / ".env"
    if root_env.exists() and not wt_env.exists():
        try:
            shutil.copy2(root_env, wt_env)
        except Exception:
            pass

    print(f"[Plane1Worktree] [SUCCESS] Sandbox created successfully:")
    print(f"  - Worktree Path : {wt_path}")
    print(f"  - Branch Name   : {branch}")
    print(f"  - Base Reference: {base_ref}")
    print(f"\nAgents may now execute edits exclusively inside: {wt_path}")
    return 0

def test_worktree(task_slug: str, skip_rust: bool = False) -> int:
    """Execute plane1_fast_gate inside the target worktree sandbox."""
    slug = validate_slug(task_slug)
    wt_path = get_worktree_path(slug)

    if not wt_path.exists():
        print(f"[Plane1Worktree] ERROR: Worktree '{wt_path}' does not exist.")
        return 1

    print(f"\n[Plane1Worktree] Executing Fast-Gate inside sandbox: {wt_path}")
    fast_gate_script = PROJECT_ROOT / "execution" / "plane1_fast_gate.py"
    
    cmd = [sys.executable, str(fast_gate_script)]
    if skip_rust:
        cmd.append("--skip-rust")

    proc = subprocess.run(cmd, cwd=str(wt_path))
    return proc.returncode

def merge_worktree(task_slug: str, squash: bool = True, skip_test: bool = False) -> int:
    """Verify and merge changes from worktree branch back into main branch."""
    slug = validate_slug(task_slug)
    wt_path = get_worktree_path(slug)
    branch = get_branch_name(slug)

    if not wt_path.exists():
        print(f"[Plane1Worktree] ERROR: Worktree '{wt_path}' does not exist.")
        return 1

    # Step 1: Pre-merge verification gate
    if not skip_test:
        print(f"[Plane1Worktree] Running pre-merge verification gate...")
        code = test_worktree(slug)
        if code != 0:
            print(f"[Plane1Worktree] [ABORT] Fast-Gate failed in sandbox. Refusing to merge.")
            return code

    # Step 2: Commit any unstaged changes in the worktree
    status = run_git(["status", "--porcelain"], cwd=wt_path, check=False)
    if status.stdout.strip():
        print(f"[Plane1Worktree] Staging and committing uncommitted changes in worktree...")
        run_git(["add", "-A"], cwd=wt_path)
        commit_msg = f"plane1({slug}): automated sovereign codebase lifecycle update"
        run_git(["commit", "-m", commit_msg], cwd=wt_path)

    # Step 3: Merge back to main repository
    print(f"[Plane1Worktree] Merging branch '{branch}' into root repository...")
    if squash:
        merge_proc = run_git(["merge", "--squash", branch], check=False)
    else:
        merge_proc = run_git(["merge", branch], check=False)

    if merge_proc.returncode != 0:
        print(f"[Plane1Worktree] [CONFLICT] Merge failed:\n{merge_proc.stderr}")
        return 1

    # Finalize squash commit if squash was used
    if squash:
        commit_msg = f"feat(plane1): merge verified task '{slug}'"
        run_git(["commit", "-m", commit_msg], check=False)

    # Step 4: Cleanup worktree and branch
    print(f"[Plane1Worktree] Cleaning up sandbox...")
    abort_worktree(slug)
    print(f"[Plane1Worktree] [SUCCESS] Task '{slug}' merged and sandbox removed cleanly.")
    return 0

def abort_worktree(task_slug: str) -> int:
    """Forcefully purge the worktree sandbox and delete the temporary branch."""
    slug = validate_slug(task_slug)
    wt_path = get_worktree_path(slug)
    branch = get_branch_name(slug)

    print(f"[Plane1Worktree] Aborting sandbox for task: '{slug}'...")

    # Remove worktree through git
    try:
        run_git(["worktree", "remove", "--force", str(wt_path)], check=False)
    except Exception:
        pass

    # If directory still lingers on disk, force delete it
    if wt_path.exists():
        try:
            shutil.rmtree(wt_path, ignore_errors=True)
        except Exception as e:
            print(f"[Plane1Worktree] Warning: failed to delete directory: {e}")

    # Prune git worktree records
    try:
        run_git(["worktree", "prune"], check=False)
    except Exception:
        pass

    # Delete the branch
    try:
        run_git(["branch", "-D", branch], check=False)
    except Exception:
        pass

    print(f"[Plane1Worktree] [CLEAN] Worktree and branch '{branch}' purged.")
    return 0

def list_worktrees() -> int:
    """List all active Plane 1 worktree sandboxes."""
    proc = run_git(["worktree", "list", "--porcelain"], check=False)
    if proc.returncode != 0:
        print(f"[Plane1Worktree] ERROR: Failed to list worktrees: {proc.stderr}")
        return 1

    lines = proc.stdout.strip().splitlines()
    worktrees = []
    current: Dict[str, str] = {}
    
    for line in lines:
        if line.startswith("worktree "):
            if current:
                worktrees.append(current)
            current = {"path": line.split(" ", 1)[1].strip()}
        elif line.startswith("branch "):
            current["branch"] = line.split(" ", 1)[1].strip()
        elif line.startswith("HEAD "):
            current["head"] = line.split(" ", 1)[1].strip()
    if current:
        worktrees.append(current)

    plane1_wts = [w for w in worktrees if ".tmp/worktrees" in w.get("path", "").replace("\\", "/")]

    print("\n==================================================================")
    print(f"  ACTIVE PLANE 1 WORKTREE SANDBOXES ({len(plane1_wts)})")
    print("==================================================================")
    if not plane1_wts:
        print("  No active Plane 1 sandboxes found.")
    else:
        for w in plane1_wts:
            branch_short = w.get("branch", "detached").replace("refs/heads/", "")
            print(f"  - Branch: {branch_short:<25} Path: {w['path']}")
    print("==================================================================\n")
    return 0

def main():
    parser = argparse.ArgumentParser(description="Plane 1 Git Worktree Sandbox Manager")
    subparsers = parser.add_subparsers(dest="command", required=True)

    # create
    p_create = subparsers.add_parser("create", help="Create new worktree sandbox")
    p_create.add_argument("slug", help="Task slug (e.g., fix-agent-routing)")
    p_create.add_argument("--base", default="HEAD", help="Base git commit/branch (default: HEAD)")

    # test
    p_test = subparsers.add_parser("test", help="Run fast-gate inside worktree")
    p_test.add_argument("slug", help="Task slug")
    p_test.add_argument("--skip-rust", action="store_true", help="Skip Rust compilation")

    # merge
    p_merge = subparsers.add_parser("merge", help="Verify and merge worktree into main")
    p_merge.add_argument("slug", help="Task slug")
    p_merge.add_argument("--no-squash", dest="squash", action="store_false", help="Preserve individual commits")
    p_merge.add_argument("--skip-test", action="store_true", help="Skip pre-merge verification")

    # abort
    p_abort = subparsers.add_parser("abort", help="Discard worktree and purge branch")
    p_abort.add_argument("slug", help="Task slug")

    # list
    subparsers.add_parser("list", help="List active sandboxes")

    args = parser.parse_args()

    try:
        if args.command == "create":
            sys.exit(create_worktree(args.slug, args.base))
        elif args.command == "test":
            sys.exit(test_worktree(args.slug, args.skip_rust))
        elif args.command == "merge":
            sys.exit(merge_worktree(args.slug, args.squash, args.skip_test))
        elif args.command == "abort":
            sys.exit(abort_worktree(args.slug))
        elif args.command == "list":
            sys.exit(list_worktrees())
    except Exception as e:
        print(f"[Plane1Worktree] ERROR: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main()

