/**
 * ### AI Assist Note (Knowledge Heritage):
 * - @docs ARCHITECTURE:Core:Skills
 * - Failure Path: Missing context, instructions drift
 * - Telemetry Link: Search [pollywog-mcp] in system logs
 *
 * ### AI Assist Note
 * Pollywog Model Context Protocol (MCP) server that parses clean-code skills to serve tools & prompts.
 *
 * ### 🔍 Debugging & Observability
 * Traceability via stdio transport logs.
 */

import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const SKILL_PATH = path.join(__dirname, "..", "skills", "clean-code", "SKILL.md");

function getPollywogInstructions() {
  try {
    const content = fs.readFileSync(SKILL_PATH, "utf8");
    const marker = "## Pollywog AI Minimalism & Safety";
    const startIndex = content.indexOf(marker);
    if (startIndex === -1) {
      return "Pollywog instructions marker not found in clean-code skill.";
    }
    
    // Find the next --- separator after the marker
    const searchArea = content.substring(startIndex);
    const endMatch = searchArea.match(/\n\s*---\s*\n/);
    if (endMatch) {
      return searchArea.substring(0, endMatch.index).trim();
    }
    return searchArea.trim();
  } catch (e) {
    return "Error reading Pollywog instructions: " + e.message;
  }
}

const server = new McpServer({ name: "pollywog", version: "1.0.0" });

server.registerPrompt(
  "pollywog",
  {
    title: "Pollywog Mode",
    description: "Serve the Pollywog AI minimalism instructions (YAGNI, standard library first, safety boundaries)."
  },
  () => ({
    messages: [{ role: "user", content: { type: "text", text: getPollywogInstructions() } }],
  })
);

server.registerTool(
  "pollywog_instructions",
  {
    title: "Pollywog Instructions",
    description: "Get the latest Pollywog instructions parsed from the clean-code skill."
  },
  () => {
    const instructions = getPollywogInstructions();
    return {
      content: [{ type: "text", text: instructions }]
    };
  }
);

await server.connect(new StdioServerTransport());
