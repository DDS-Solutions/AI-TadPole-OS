"""
@docs ARCHITECTURE:Core, ARCHITECTURE:Sandboxing

### AI Context Alignment
- **Subsystem**: Sovereign Engine / Plane 1 Sandboxing
- **Primary Entrypoints**: `main`, `create_worktree`, `test_worktree`, `merge_worktree`, `abort_worktree`, `list_worktrees`

### ⚠️ Non-Negotiables
- `[Structural]` Deterministic worktree sandboxing under .tmp/worktrees/ with zero root working tree pollution.
- `[Structural]` Safe rollback: abort unconditionally purges task branch without leaving debris or touching root node_modules.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: `[Plane1Worktree]`
"""

import os
import sys
import stat
import shutil
import argparse
import subprocess
from pathlib import Path
from typing import List, Dict, Optional, Set

PROJECT_ROOT = Path(__file__).resolve().parent.parent
WORKTREES_ROOT = PROJECT_ROOT / ".tmp" / "worktrees"
ACTIVE_WORKTREE_PATH = WORKTREES_ROOT / "_active"
SHARED_CARGO_TARGET = PROJECT_ROOT / ".tmp" / "plane1-cargo-target"


def build_sandbox_env(base_env: Optional[Dict[str, str]] = None) -> Dict[str, str]:
    """Provide a shared sandbox CARGO_TARGET_DIR to avoid cold builds and live server-rs.exe locks."""
    env = dict(base_env if base_env is not None else os.environ)
    if not env.get("CARGO_TARGET_DIR"):
        SHARED_CARGO_TARGET.mkdir(parents=True, exist_ok=True)
        env["CARGO_TARGET_DIR"] = str(SHARED_CARGO_TARGET)
    return env


def run_git(args: List[str], cwd: Path = PROJECT_ROOT, check: bool = True) -> subprocess.CompletedProcess:
    """Run git command (with fsmonitor disabled to avoid Windows pipe hangs) and return CompletedProcess."""
    return subprocess.run(
        ["git", "-c", "core.fsmonitor=false"] + args,
        cwd=str(cwd),
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        check=check,
    )


def validate_slug(task_slug: str) -> str:
    """Ensure task slug is safe for branch and directory names."""
    slug = task_slug.strip()
    if not slug or len(slug) > 64:
        raise ValueError("Task slug must be non-empty and <= 64 characters.")
    if slug == "_active":
        raise ValueError("Task slug '_active' is reserved for the reusable sandbox slot.")
    if not all(c.isalnum() or c in ("-", "_") for c in slug):
        raise ValueError(f"Task slug '{slug}' contains invalid characters. Use alphanumeric, hyphens, and underscores only.")
    return slug


def get_worktree_path(task_slug: str, isolated: bool = False) -> Path:
    """Return target worktree path (reusable _active slot by default to preserve Cargo incremental cache)."""
    return (WORKTREES_ROOT / task_slug) if isolated else ACTIVE_WORKTREE_PATH


def get_branch_name(task_slug: str) -> str:
    return f"plane1/{task_slug}"


def _parse_worktree_list() -> List[Dict[str, str]]:
    proc = run_git(["worktree", "list", "--porcelain"], check=False)
    if proc.returncode != 0:
        return []

    worktrees: List[Dict[str, str]] = []
    current: Dict[str, str] = {}
    for line in (proc.stdout or "").splitlines():
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
    return worktrees


def resolve_worktree_path(task_slug: str) -> Optional[Path]:
    """Locate the active worktree directory hosting `plane1/<task_slug>` (either `_active` or `<slug>`)."""
    target_ref = f"refs/heads/{get_branch_name(task_slug)}"
    for wt in _parse_worktree_list():
        if wt.get("branch") == target_ref:
            wt_path = Path(wt["path"])
            if wt_path.exists():
                return wt_path

    legacy_path = WORKTREES_ROOT / task_slug
    if legacy_path.exists():
        return legacy_path
    return None


def _is_link_or_junction(path: Path) -> bool:
    """Return True if path is a POSIX symlink or Windows NTFS junction/reparse point."""
    if os.path.islink(path):
        return True
    if os.name == "nt":
        isjunction = getattr(os.path, "isjunction", None)
        if callable(isjunction) and isjunction(path):
            return True
        try:
            st = os.lstat(path)
            return bool(getattr(st, "st_file_attributes", 0) & stat.FILE_ATTRIBUTE_REPARSE_POINT)
        except OSError:
            return False
    return False


def link_node_modules(wt_path: Path) -> None:
    """Junction/symlink root node_modules into the sandbox so Vitest and linters work natively."""
    root_nm = PROJECT_ROOT / "node_modules"
    wt_nm = wt_path / "node_modules"
    if not root_nm.is_dir() or wt_nm.exists() or _is_link_or_junction(wt_nm):
        return

    try:
        if os.name == "nt":
            subprocess.run(
                ["cmd", "/c", "mklink", "/J", str(wt_nm), str(root_nm)],
                capture_output=True,
                check=False,
            )
        else:
            os.symlink(root_nm, wt_nm, target_is_directory=True)
    except OSError as exc:
        print(f"[Plane1Worktree] Warning: failed to link node_modules: {exc}")


def unlink_node_modules(wt_path: Path) -> None:
    """Safely remove a sandbox node_modules junction/symlink without touching root node_modules."""
    wt_nm = wt_path / "node_modules"
    if not _is_link_or_junction(wt_nm):
        return

    try:
        if os.name == "nt":
            os.rmdir(wt_nm)
        else:
            wt_nm.unlink()
    except OSError as exc:
        print(f"[Plane1Worktree] Warning: failed to unlink node_modules junction: {exc}")


def _sync_sandbox_assets(wt_path: Path) -> None:
    """Copy root .env (if present) and link root node_modules into the worktree."""
    root_env = PROJECT_ROOT / ".env"
    wt_env = wt_path / ".env"
    if root_env.exists() and not wt_env.exists():
        try:
            shutil.copy2(root_env, wt_env)
        except OSError as exc:
            print(f"[Plane1Worktree] Warning: failed to copy .env into sandbox: {exc}")
    link_node_modules(wt_path)


def _reuse_active_worktree(branch: str, base_ref: str) -> int:
    """Switch the reusable `_active` worktree onto a fresh branch if it has no uncommitted edits."""
    status = run_git(["status", "--porcelain"], cwd=ACTIVE_WORKTREE_PATH, check=False)
    if (status.stdout or "").strip():
        cur_branch = run_git(["rev-parse", "--abbrev-ref", "HEAD"], cwd=ACTIVE_WORKTREE_PATH, check=False)
        active_name = (cur_branch.stdout or "unknown").strip()
        print(f"[Plane1Worktree] [ABORT] Reusable sandbox '{ACTIVE_WORKTREE_PATH}' has uncommitted changes on '{active_name}'.")
        print("[Plane1Worktree] Merge/abort the active task first, or pass --isolated for a parallel worktree.")
        return 1

    checkout = run_git(["checkout", "-B", branch, base_ref], cwd=ACTIVE_WORKTREE_PATH, check=False)
    if checkout.returncode != 0:
        print(f"[Plane1Worktree] ERROR: Failed to switch reusable sandbox:\n{checkout.stderr}")
        return 1

    _sync_sandbox_assets(ACTIVE_WORKTREE_PATH)
    return 0


def create_worktree(task_slug: str, base_ref: str = "HEAD", isolated: bool = False) -> int:
    """Create (or reuse `_active`) git worktree sandbox for `plane1/<task_slug>`."""
    slug = validate_slug(task_slug)
    wt_path = get_worktree_path(slug, isolated=isolated)
    branch = get_branch_name(slug)

    print(f"\n[Plane1Worktree] Initializing sandbox for task: '{slug}'")
    WORKTREES_ROOT.mkdir(parents=True, exist_ok=True)

    # Reuse the fixed `_active` directory when already registered so Cargo keeps warm fingerprints
    if not isolated and (ACTIVE_WORKTREE_PATH / ".git").exists():
        if _reuse_active_worktree(branch, base_ref) != 0:
            return 1
        _print_sandbox_ready(wt_path, branch, base_ref, reused=True)
        return 0

    if wt_path.exists():
        print(f"[Plane1Worktree] Worktree path '{wt_path}' already exists; cleaning previous instance...")
        _purge_worktree_dir(wt_path, branch)

    proc = run_git(["branch", "--list", branch], check=False)
    if branch in (proc.stdout or ""):
        print(f"[Plane1Worktree] Pruning stale branch '{branch}'...")
        run_git(["branch", "-D", branch], check=False)

    proc = run_git(["worktree", "add", "-b", branch, str(wt_path), base_ref], check=False)
    if proc.returncode != 0:
        print(f"[Plane1Worktree] ERROR: Failed to create worktree:\n{proc.stderr}")
        return 1

    _sync_sandbox_assets(wt_path)
    _print_sandbox_ready(wt_path, branch, base_ref, reused=False)
    return 0


def _print_sandbox_ready(wt_path: Path, branch: str, base_ref: str, reused: bool) -> None:
    mode = "reused warm slot" if reused else "created"
    print(f"[Plane1Worktree] [SUCCESS] Sandbox {mode}:")
    print(f"  - Worktree Path : {wt_path}")
    print(f"  - Branch Name   : {branch}")
    print(f"  - Base Reference: {base_ref}")
    print(f"\nAgents may now execute edits exclusively inside: {wt_path}")


def test_worktree(task_slug: str, skip_rust: bool = False, strict: bool = False) -> int:
    """Execute plane1_fast_gate inside the target worktree sandbox."""
    slug = validate_slug(task_slug)
    wt_path = resolve_worktree_path(slug)

    if wt_path is None:
        print(f"[Plane1Worktree] ERROR: Worktree for task '{slug}' does not exist.")
        return 1

    print(f"\n[Plane1Worktree] Executing Fast-Gate inside sandbox: {wt_path}")
    wt_fast_gate = wt_path / "execution" / "plane1_fast_gate.py"
    fast_gate_script = wt_fast_gate if wt_fast_gate.exists() else (PROJECT_ROOT / "execution" / "plane1_fast_gate.py")

    cmd = [sys.executable, str(fast_gate_script)]
    if skip_rust:
        cmd.append("--skip-rust")
    if strict:
        cmd.append("--strict")

    proc = subprocess.run(cmd, cwd=str(wt_path), env=build_sandbox_env())
    return proc.returncode


def check_root_clean() -> Optional[str]:
    """Return tracked dirty status string if root repo has uncommitted changes, else None."""
    proc = run_git(["status", "--porcelain", "--untracked-files=no"], cwd=PROJECT_ROOT, check=False)
    dirty = (proc.stdout or "").strip()
    return dirty if dirty else None


def parse_porcelain_paths(porcelain_output: str) -> Set[str]:
    """Extract normalized relative file paths from `git status --porcelain` output."""
    paths: Set[str] = set()
    for line in porcelain_output.splitlines():
        if len(line) < 4:
            continue
        raw = line[3:].strip()
        if " -> " in raw:
            raw = raw.split(" -> ", 1)[1].strip()
        paths.add(raw.strip('"').replace("\\", "/"))
    return paths


def _collect_sandbox_changed_paths(wt_path: Path, branch: str) -> Set[str]:
    """Collect all paths modified or added in the sandbox (both committed and uncommitted)."""
    changed: Set[str] = set()
    wt_status = run_git(["status", "--porcelain"], cwd=wt_path, check=False)
    changed.update(parse_porcelain_paths(wt_status.stdout or ""))

    diff_proc = run_git(["diff", "--name-only", f"HEAD...{branch}"], cwd=PROJECT_ROOT, check=False)
    for line in (diff_proc.stdout or "").splitlines():
        if line.strip():
            changed.add(line.strip().replace("\\", "/"))
    return changed


def _prepare_root_for_merge(
    slug: str, wt_path: Path, branch: str, stash_root: bool
) -> tuple[int, bool]:
    """Ensure root is merge-ready; optionally stash non-overlapping dirty tracked files."""
    dirty_root = check_root_clean()
    if not dirty_root:
        return 0, False

    if not stash_root:
        print(f"[Plane1Worktree] [ABORT] Root repository has uncommitted tracked changes:\n{dirty_root}")
        print("[Plane1Worktree] Commit, stash, or pass --stash-root for non-overlapping root edits.")
        return 1, False

    dirty_paths = parse_porcelain_paths(dirty_root)
    overlap = dirty_paths & _collect_sandbox_changed_paths(wt_path, branch)
    if overlap:
        print(f"[Plane1Worktree] [ABORT] Root dirty files overlap with sandbox changes: {sorted(overlap)}")
        return 1, False

    stash_proc = run_git(
        ["stash", "push", "-m", f"plane1-merge-{slug}", "--", *sorted(dirty_paths)],
        cwd=PROJECT_ROOT,
        check=False,
    )
    if stash_proc.returncode != 0:
        print(f"[Plane1Worktree] [ABORT] Failed to auto-stash root changes:\n{stash_proc.stderr}")
        return 1, False
    print(f"[Plane1Worktree] Auto-stashed non-overlapping root file(s): {', '.join(sorted(dirty_paths))}")
    return 0, True


def merge_worktree(
    task_slug: str,
    squash: bool = True,
    skip_test: bool = False,
    strict: bool = False,
    stash_root: bool = False,
) -> int:
    """Verify and merge changes from worktree branch back into main branch."""
    slug = validate_slug(task_slug)
    wt_path = resolve_worktree_path(slug)
    branch = get_branch_name(slug)

    if wt_path is None:
        print(f"[Plane1Worktree] ERROR: Worktree for task '{slug}' does not exist.")
        return 1

    prep_rc, stashed = _prepare_root_for_merge(slug, wt_path, branch, stash_root)
    if prep_rc != 0:
        return prep_rc

    try:
        return _execute_merge_steps(slug, wt_path, branch, squash, skip_test, strict)
    finally:
        if stashed:
            pop_proc = run_git(["stash", "pop"], cwd=PROJECT_ROOT, check=False)
            if pop_proc.returncode != 0:
                print(f"[Plane1Worktree] Warning: git stash pop failed:\n{pop_proc.stderr or pop_proc.stdout}")


def _execute_merge_steps(
    slug: str,
    wt_path: Path,
    branch: str,
    squash: bool,
    skip_test: bool,
    strict: bool,
) -> int:
    if not skip_test:
        print("[Plane1Worktree] Running pre-merge verification gate...")
        code = test_worktree(slug, strict=strict)
        if code != 0:
            print("[Plane1Worktree] [ABORT] Fast-Gate failed in sandbox. Refusing to merge.")
            return code

    status = run_git(["status", "--porcelain"], cwd=wt_path, check=False)
    if (status.stdout or "").strip():
        print("[Plane1Worktree] Staging and committing uncommitted changes in worktree...")
        run_git(["add", "-A"], cwd=wt_path)
        commit_msg = f"plane1({slug}): automated sovereign codebase lifecycle update"
        commit_proc = run_git(["commit", "-m", commit_msg], cwd=wt_path, check=False)
        if commit_proc.returncode != 0:
            print(f"[Plane1Worktree] [ABORT] Failed to commit sandbox changes:\n{commit_proc.stderr or commit_proc.stdout}")
            return 1

    print(f"[Plane1Worktree] Merging branch '{branch}' into root repository...")
    merge_args = ["merge", "--squash", branch] if squash else ["merge", branch]
    merge_proc = run_git(merge_args, check=False)
    if merge_proc.returncode != 0:
        print(f"[Plane1Worktree] [CONFLICT] Merge failed:\n{merge_proc.stderr or merge_proc.stdout}")
        return 1

    if squash:
        squash_commit = run_git(["commit", "-m", f"feat(plane1): merge verified task '{slug}'"], check=False)
        if squash_commit.returncode != 0:
            print(
                "[Plane1Worktree] [ABORT] Squash commit failed (sandbox preserved for recovery):\n"
                f"{squash_commit.stderr or squash_commit.stdout}"
            )
            return 1

    print("[Plane1Worktree] Cleaning up sandbox...")
    abort_worktree(slug)
    print(f"[Plane1Worktree] [SUCCESS] Task '{slug}' merged and sandbox cleaned.")
    return 0


def _purge_worktree_dir(wt_path: Path, branch: str) -> None:
    """Safely unlink node_modules junction before removing a worktree directory and branch."""
    unlink_node_modules(wt_path)
    run_git(["worktree", "remove", "--force", str(wt_path)], check=False)
    if wt_path.exists():
        shutil.rmtree(wt_path, ignore_errors=True)
    run_git(["worktree", "prune"], check=False)
    run_git(["branch", "-D", branch], check=False)


def abort_worktree(task_slug: str, purge_dir: bool = False) -> int:
    """Reset the reusable `_active` slot (or purge an isolated worktree) and delete the task branch."""
    slug = validate_slug(task_slug)
    wt_path = resolve_worktree_path(slug)
    branch = get_branch_name(slug)

    print(f"[Plane1Worktree] Aborting sandbox for task: '{slug}'...")

    is_active_slot = wt_path is not None and wt_path.resolve() == ACTIVE_WORKTREE_PATH.resolve()
    if is_active_slot and not purge_dir:
        run_git(["reset", "--hard", "HEAD"], cwd=wt_path, check=False)
        run_git(["clean", "-fd"], cwd=wt_path, check=False)
        run_git(["checkout", "--detach", "HEAD"], cwd=wt_path, check=False)
        run_git(["branch", "-D", branch], check=False)
        print(f"[Plane1Worktree] [CLEAN] Branch '{branch}' deleted; warm slot '{ACTIVE_WORKTREE_PATH}' retained.")
        return 0

    target_dir = wt_path if wt_path is not None else (WORKTREES_ROOT / slug)
    _purge_worktree_dir(target_dir, branch)
    print(f"[Plane1Worktree] [CLEAN] Worktree and branch '{branch}' purged.")
    return 0


def list_worktrees() -> int:
    """List all active Plane 1 worktree sandboxes."""
    worktrees = _parse_worktree_list()
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

    p_create = subparsers.add_parser("create", help="Create or reuse worktree sandbox")
    p_create.add_argument("slug", help="Task slug (e.g., fix-agent-routing)")
    p_create.add_argument("--base", default="HEAD", help="Base git commit/branch (default: HEAD)")
    p_create.add_argument("--isolated", action="store_true", help="Use a dedicated .tmp/worktrees/<slug> path instead of _active")

    p_test = subparsers.add_parser("test", help="Run fast-gate inside worktree")
    p_test.add_argument("slug", help="Task slug")
    p_test.add_argument("--skip-rust", action="store_true", help="Skip Rust compilation")
    p_test.add_argument("--strict", action="store_true", help="Fail on any warning")

    p_merge = subparsers.add_parser("merge", help="Verify and merge worktree into main")
    p_merge.add_argument("slug", help="Task slug")
    p_merge.add_argument("--no-squash", dest="squash", action="store_false", help="Preserve individual commits")
    p_merge.add_argument("--skip-test", action="store_true", help="Skip pre-merge verification")
    p_merge.add_argument("--strict", action="store_true", help="Fail on any warning")
    p_merge.add_argument("--stash-root", action="store_true", help="Auto-stash non-overlapping dirty root files during merge")

    p_abort = subparsers.add_parser("abort", help="Discard worktree changes and delete branch")
    p_abort.add_argument("slug", help="Task slug")
    p_abort.add_argument("--purge", action="store_true", help="Also remove the reusable _active worktree directory")

    subparsers.add_parser("list", help="List active sandboxes")

    args = parser.parse_args()

    try:
        if args.command == "create":
            sys.exit(create_worktree(args.slug, args.base, isolated=args.isolated))
        elif args.command == "test":
            sys.exit(test_worktree(args.slug, args.skip_rust, args.strict))
        elif args.command == "merge":
            sys.exit(merge_worktree(args.slug, args.squash, args.skip_test, args.strict, stash_root=args.stash_root))
        elif args.command == "abort":
            sys.exit(abort_worktree(args.slug, purge_dir=args.purge))
        elif args.command == "list":
            sys.exit(list_worktrees())
    except Exception as e:
        print(f"[Plane1Worktree] ERROR: {e}")
        sys.exit(1)


if __name__ == "__main__":
    main()
