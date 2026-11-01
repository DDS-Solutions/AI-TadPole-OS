-- Migration: Add mcp_tools column to agents
-- Supports assigning MCP tools to specific agents.

ALTER TABLE agents ADD COLUMN mcp_tools TEXT;
