/**
 * @docs ARCHITECTURE:Infrastructure
 * 
 * ### AI Assist Note
 * **Skill Parser**: Utility for deserializing agent capabilities from Markdown files.
 * Supports extracting metadata from JSON blocks or structured headers.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Malformed JSON in markdown block (parse error), missing `#` header in unstructured files (fallback name assigned), or ID sanitization collision.
 * - **Telemetry Link**: Search for `SkillParser` in UI logs or capability import traces.
 */

import { ValidationUtils } from './validation_utils';
import type { Skill_Definition, Workflow_Definition, Hook_Definition } from '../stores/skill_store';

export interface ParsedCapability {
    type: 'skill' | 'workflow' | 'hook';
    data: Skill_Definition | Workflow_Definition | Hook_Definition;
}

export const SkillParser = {
    /**
     * Parses a .md file content to extract a skill or workflow definition.
     */
    parse_markdown: (content: string): ParsedCapability | null => {
        // 1. Try to find a JSON block for skills/hooks
        const json_match = content.match(/```json\s*([\s\S]*?)\s*```/);
        if (json_match) {
            try {
                const data = JSON.parse(json_match[1]);
                if (data.execution_command) {
                    return { type: 'skill', data };
                }
                if (data.hook_type) {
                    return { type: 'hook', data };
                }
            } catch (e) {
                console.error("Failed to parse JSON block in skill markdown:", e);
            }
        }

        // 2. Fallback to extracting from text (structured headers)
        const name_match = content.match(/# (.*)/);
        const desc_match = content.match(/Description: (.*)/i);
        const cmd_match = content.match(/Command: (.*)/i);
        
        const name = name_match ? name_match[1].trim() : "unnamed_imported_skill";
        const description = desc_match ? desc_match[1].trim() : "";
        const execution_command = cmd_match ? cmd_match[1].trim() : "";

        // If we have an execution command, it's likely a skill
        if (execution_command) {
            return {
                type: 'skill',
                data: {
                    name: ValidationUtils.sanitize_id(name),
                    description,
                    execution_command,
                    schema: {}, // Default empty schema
                    category: 'user'
                } as Skill_Definition
            };
        }

        // Otherwise, treat as a workflow
        return {
            type: 'workflow',
            data: {
                name: ValidationUtils.sanitize_id(name),
                content: content.replace(/# .*/, '').trim(), // Use the rest as content
                category: 'user'
            } as Workflow_Definition
        };
    }
};
