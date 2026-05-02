/**
 * @docs ARCHITECTURE:TestSuites
 * 
 * ### AI Assist Note
 * **Verification Suite**: Skill Parser Logic. 
 * Validates the extraction and sanitization of agent capabilities from Markdown and JSON blocks. 
 * Ensures robust fallback behavior for malformed inputs and unstructured documentation.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Regex failure in pattern extraction or sanitization collision in unnamed skills.
 * - **Telemetry Link**: Run `npm run test` or search `[skill_parser.test]` in Vitest logs.
 */

import { describe, it, expect, vi } from 'vitest';
import { SkillParser } from './skill_parser';

describe('SkillParser', () => {
    it('parses a skill from a JSON markdown block', () => {
        const content = `
# My Skill
This is a skill.
\`\`\`json
{
    "name": "test-skill",
    "description": "A test skill",
    "execution_command": "echo hello",
    "schema": {}
}
\`\`\`
        `;
        const result = SkillParser.parse_markdown(content);
        expect(result).not.toBeNull();
        expect(result?.type).toBe('skill');
        expect(result?.data.name).toBe('test-skill');
        expect((result?.data as any).execution_command).toBe('echo hello');
    });

    it('parses a hook from a JSON markdown block', () => {
        const content = `
\`\`\`json
{
    "name": "test-hook",
    "hook_type": "pre_execution",
    "target": "all"
}
\`\`\`
        `;
        const result = SkillParser.parse_markdown(content);
        expect(result).not.toBeNull();
        expect(result?.type).toBe('hook');
        expect((result?.data as any).hook_type).toBe('pre_execution');
    });

    it('falls back to header parsing if no JSON block is present', () => {
        const content = `
# Custom Skill
Description: A manually defined skill.
Command: ls -la
        `;
        const result = SkillParser.parse_markdown(content);
        expect(result).not.toBeNull();
        expect(result?.type).toBe('skill');
        expect(result?.data.name).toBe('custom-skill'); // Sanitized
        expect(result?.data.description).toBe('A manually defined skill.');
        expect((result?.data as any).execution_command).toBe('ls -la');
    });

    it('treats content without a command as a workflow', () => {
        const content = `
# My Workflow
Step 1: Do something.
Step 2: Do something else.
        `;
        const result = SkillParser.parse_markdown(content);
        expect(result).not.toBeNull();
        expect(result?.type).toBe('workflow');
        expect(result?.data.name).toBe('my-workflow');
        expect((result?.data as any).content).toContain('Step 1');
    });

    it('handles malformed JSON gracefully', () => {
        const console_spy = vi.spyOn(console, 'error').mockImplementation(() => {});
        const content = `
\`\`\`json
{ "invalid": json
\`\`\`
# Recovered Skill
Command: recovery
        `;
        const result = SkillParser.parse_markdown(content);
        expect(result).not.toBeNull();
        expect(result?.type).toBe('skill');
        expect(result?.data.name).toBe('recovered-skill');
        expect(console_spy).toHaveBeenCalled();
        console_spy.mockRestore();
    });

    it('provides a default name for completely unparseable content', () => {
        const result = SkillParser.parse_markdown("");
        expect(result).not.toBeNull();
        expect(result?.data.name).toBe('unnamed-imported-skill');
    });
});

// Metadata: [skill_parser_test]

// Metadata: [skill_parser_test]
