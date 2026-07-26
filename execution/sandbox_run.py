#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Infrastructure:Sandbox

### AI Assist Note
**🚀 Tadpole Engine: Git Worktree Sandbox Runner**
Safely check out and run test commands in an isolated Copy-on-Write Git worktree environment.
Shares caching layers to keep execution times fast.

### 🔍 Debugging & Observability
- **Failure Path**: Git command failures, permission errors, disk space limits, or symlink failures.
- **Telemetry Link**: Search `[sandbox_run]` in logs.
"""

import os
import sys
import shutil
import subprocess
import tempfile
import uuid
from pathlib import Path

def create_junction(src: Path, dst: Path):
    """Creates a Windows directory junction or unix symlink (non-admin safe)"""
    if os.name == 'nt':
        # Use Windows built-in mklink /j which does not require administrator rights
        cmd = ["cmd", "/c", "mklink", "/j", str(dst), str(src)]
        subprocess.run(cmd, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, check=True)
    else:
        os.symlink(src, dst, target_is_directory=True)

def main():
    if len(sys.argv) < 2:
        print("Usage: python execution/sandbox_run.py <command_to_run_in_sandbox>")
        sys.exit(1)

    command_to_run = sys.argv[1]
    print(f"[sandbox_run] Executing command in sandbox: {command_to_run}")

    # Resolve workspace paths
    workspace_root = Path(__file__).resolve().parent.parent
    tmp_dir = workspace_root / ".tmp"
    tmp_dir.mkdir(exist_ok=True)

    # Unique sandbox ID
    sandbox_id = f"sandbox_{uuid.uuid4().hex[:8]}"
    sandbox_path = tmp_dir / sandbox_id

    print(f"[Sandbox] Spawning isolated worktree at: {sandbox_path}")

    # Step 1: Create git worktree
    # We use --detach HEAD so we don't pollute the git branch namespace
    try:
        subprocess.run(
            ["git", "worktree", "add", "--detach", str(sandbox_path), "HEAD"],
            cwd=str(workspace_root),
            check=True,
            capture_output=True
        )
    except subprocess.CalledProcessError as e:
        print(f"[Sandbox] Failed to create git worktree: {e.stderr.decode('utf-8', errors='replace')}")
        sys.exit(1)

    cleanup_needed = True

    try:
        # Step 2: Copy unstaged/dirty changes from current workspace to sandbox
        # So we can test local modifications before committing
        status_proc = subprocess.run(
            ["git", "status", "--porcelain"],
            cwd=str(workspace_root),
            capture_output=True,
            check=True
        )
        status_lines = status_proc.stdout.decode('utf-8', errors='replace').splitlines()
        
        for line in status_lines:
            if len(line) < 4:
                continue
            status_code = line[:2]
            rel_file_path = line[3:]
            
            # Source & dest paths
            src_file = workspace_root / rel_file_path
            dst_file = sandbox_path / rel_file_path
            
            # Skip untracked/ignored folders (like .tmp itself or node_modules)
            if any(part in src_file.parts for part in [".tmp", "node_modules", "target", ".git"]):
                continue

            if status_code in (" M", "M ", "A ", " A"):
                if src_file.is_file():
                    dst_file.parent.mkdir(parents=True, exist_ok=True)
                    shutil.copy2(src_file, dst_file)
            elif status_code in (" D", "D "):
                if dst_file.is_file():
                    dst_file.unlink()

        # Step 3: Resource sharing
        # Symlink node_modules to avoid expensive npm install
        main_node_modules = workspace_root / "node_modules"
        sandbox_node_modules = sandbox_path / "node_modules"
        if main_node_modules.exists():
            create_junction(main_node_modules, sandbox_node_modules)

        # Copy local .env file if it exists
        main_env = workspace_root / ".env"
        sandbox_env = sandbox_path / ".env"
        if main_env.is_file():
            shutil.copy2(main_env, sandbox_env)

        # Step 4: Configure environment variables for Cargo target directory sharing
        env = os.environ.copy()
        main_cargo_target = workspace_root / "server-rs" / "target"
        if main_cargo_target.exists():
            env["CARGO_TARGET_DIR"] = str(main_cargo_target.resolve())

        # Step 5: Execute command in the sandbox directory
        print(f"[Sandbox] Executing command: {command_to_run}")
        
        # Use shell execution so developers can pass complex commands (e.g. "npm run test" or "cargo check")
        result = subprocess.run(
            command_to_run,
            shell=True,
            cwd=str(sandbox_path),
            env=env
        )

        exit_code = result.returncode

    except Exception as e:
        print(f"[Sandbox] Exception during execution: {e}")
        exit_code = 1

    finally:
        # Step 6: Cleanup worktree
        if cleanup_needed:
            print(f"[Sandbox] Cleaning up worktree at: {sandbox_path}")
            # Windows needs junctions removed first to prevent git worktree prune from choking
            sandbox_node_modules = sandbox_path / "node_modules"
            if sandbox_node_modules.exists():
                if os.name == 'nt':
                    # Remove junction safely using rmdir
                    subprocess.run(["cmd", "/c", "rmdir", str(sandbox_node_modules)], check=False)
                else:
                    sandbox_node_modules.unlink(missing_ok=True)
            
            # Prune git worktree
            subprocess.run(
                ["git", "worktree", "remove", "--force", str(sandbox_path)],
                cwd=str(workspace_root),
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
                check=False
            )
            
            # Clean up the branch
            subprocess.run(
                ["git", "worktree", "prune"],
                cwd=str(workspace_root),
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
                check=False
            )

    sys.exit(exit_code)

if __name__ == "__main__":
    main()
