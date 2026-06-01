> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Core**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Search `[github_scout]` in audit logs.
>
> ### AI Assist Note
> Core technical resource for the Tadpole OS Sovereign infrastructure.
>
> ### 🔍 Debugging & Observability
> Traceability via `parity_guard.py`.

---
name: github-scout
description: Scouting and analysis for GitHub repositories.
allowed-tools: mcp_github-mcp-server_search_repositories, mcp_github-mcp-server_get_file_contents
version: 1.0.0
---

# GitHub Scouting Workflow

1.  **Search Phase**: Use `github:search_repositories` to find the target repository.
2.  **Structural Analysis**: List the repository contents to understand the file structure.
3.  **Content Synthesis**: Read key files (README.md, main logic) to understand the project architecture.
4.  **Final Report**: Synthesize findings into a research dossier.
5.  **Intelligence Sharing**: Call `share_finding` with the topic "GitHub Scouting" and the research dossier as the finding.
6.  **Archiving**: Commit findings to the local workspace.

[//]: # (Metadata: [github_scout])
