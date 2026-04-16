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
5.  **Archiving**: Commit findings to the local workspace.
