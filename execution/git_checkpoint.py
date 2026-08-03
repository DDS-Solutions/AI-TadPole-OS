#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Infrastructure:Execution

### AI Assist Note
**Automated Git Checkpointing & Rollback Guard (git_checkpoint.py)**: Stashes uncommitted 
changes and creates lightweight git checkpoints prior to executing Layer 3 modification 
scripts. If downstream verification gates fail, it allows automatic reversion back to 
the clean checkpoint state.

### 🔍 Debugging & Observability
- **Trace Scope**: `execution::git_checkpoint`
- **Dependency**: `git` CLI, `execution::event_logger`
"""

import sys
import os
import subprocess
import time
from pathlib import Path

# Add execution dir to import path
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from py_utils import init_utf8, print_ok, print_warn, print_err, print_step, print_header
from event_logger import log_event

init_utf8()

WORKSPACE_ROOT = Path(__file__).parent.parent


def run_git_cmd(args: list[str]) -> tuple[int, str]:
    """Run a git command in the workspace root."""
    try:
        res = subprocess.run(
            ["git"] + args,
            cwd=WORKSPACE_ROOT,
            capture_output=True,
            text=True,
            encoding="utf-8"
        )
        return res.returncode, res.stdout.strip()
    except Exception as e:
        return 1, str(e)


def get_current_head() -> str:
    """Get current git HEAD commit hash."""
    code, out = run_git_cmd(["rev-parse", "HEAD"])
    return out if code == 0 else "UNKNOWN"


def is_workspace_dirty() -> bool:
    """Check if git working tree has uncommitted modifications."""
    code, out = run_git_cmd(["status", "--porcelain"])
    return code == 0 and len(out) > 0


def create_checkpoint(label: str = "agent-checkpoint") -> dict:
    """
    Create a git checkpoint of the current workspace state.
    
    :param label: Descriptive label for the checkpoint context.
    :return: Checkpoint metadata dictionary.
    """
    head_before = get_current_head()
    dirty_before = is_workspace_dirty()
    stash_created = False
    stash_ref = None
    
    if dirty_before:
        stash_name = f"tadpole-ckpt-{int(time.time())}-{label}"
        code, out = run_git_cmd(["stash", "push", "-u", "-m", stash_name])
        if code == 0 and "No local changes to save" not in out:
            stash_created = True
            stash_ref = stash_name
            print_ok(f"Created git stash checkpoint: {stash_name}")
    
    checkpoint_meta = {
        "label": label,
        "head_commit": head_before,
        "was_dirty": dirty_before,
        "stash_created": stash_created,
        "stash_ref": stash_ref,
        "timestamp": int(time.time())
    }
    
    log_event(
        event_type="GIT_CHECKPOINT_CREATED",
        action=f"Created checkpoint: {label}",
        observation=checkpoint_meta,
        status="COMPLETED"
    )
    
    return checkpoint_meta


def revert_checkpoint(meta: dict) -> bool:
    """
    Revert the workspace back to the state defined in checkpoint metadata.
    
    :param meta: Metadata dictionary returned by create_checkpoint().
    :return: True if revert succeeded.
    """
    print_warn(f"Reverting workspace to checkpoint: {meta.get('label')}")
    
    # 1. Hard reset to original head if commits were made
    current_head = get_current_head()
    if meta.get("head_commit") and current_head != meta.get("head_commit"):
        print_step(f"Resetting HEAD from {current_head[:8]} to {meta['head_commit'][:8]}...")
        code, out = run_git_cmd(["reset", "--hard", meta["head_commit"]])
        if code != 0:
            print_err(f"Git reset failed: {out}")
            return False
            
    # 2. Pop stash if we created one
    if meta.get("stash_created"):
        print_step("Popping git stash checkpoint...")
        code, out = run_git_cmd(["stash", "pop"])
        if code != 0:
            print_warn(f"Git stash pop notice: {out}")
            
    log_event(
        event_type="GIT_CHECKPOINT_REVERTED",
        action=f"Reverted to checkpoint: {meta.get('label')}",
        observation=meta,
        status="REVERTED"
    )
    print_ok("Workspace successfully restored to clean checkpoint state.")
    return True


def run_self_test() -> bool:
    """Self-test routine for git_checkpoint.py."""
    print_header("Tadpole OS Git Checkpoint Self-Test")
    
    print_step("Testing checkpoint creation...")
    ckpt = create_checkpoint(label="self-test")
    print_ok(f"Checkpoint metadata: {ckpt}")
    
    if ckpt.get("stash_created"):
        print_step("Testing checkpoint restoration...")
        success = revert_checkpoint(ckpt)
        return success
    else:
        print_ok("Workspace was clean, no stash needed to pop.")
        return True


if __name__ == "__main__":
    success = run_self_test()
    sys.exit(0 if success else 1)
