/**
 * @docs ARCHITECTURE:Quality:Verification
 *
 * ### AI Context Alignment
 * - **Subsystem**: Test Verification Suite / api_contracts.test
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Deterministic internal state integrity and strict interface contract compliance.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { describe, it, expect } from 'vitest';
import * as fs from 'fs';
import * as path from 'path';

// Resolve files relative to workspace root
const openapiPath = path.join(process.cwd(), 'docs', 'openapi.yaml');
const generatedPath = path.join(process.cwd(), 'src', 'contracts', 'generated.ts');

describe('API Contract & OpenAPI Alignment', () => {
    it('should find both openapi.yaml and generated.ts', () => {
        expect(fs.existsSync(openapiPath)).toBe(true);
        expect(fs.existsSync(generatedPath)).toBe(true);
    });

    const openapiContent = fs.readFileSync(openapiPath, 'utf8');
    const generatedContent = fs.readFileSync(generatedPath, 'utf8');

    describe('OpenAPI schema registration validation', () => {
        const expectedSchemas = [
            'EngineAgent',
            'RoleBlueprint',
            'TaskPayload',
            'VoiceRequest',
            'SkillManifest',
            'DangerLevel',
            'Permission'
        ];

        expectedSchemas.forEach(schemaName => {
            it(`should document ${schemaName} in openapi.yaml`, () => {
                // Look for "SchemaName:" or "$ref: '#/components/schemas/SchemaName'"
                const isDeclared = openapiContent.includes(`${schemaName}:`) || openapiContent.includes(`schemas/${schemaName}`);
                expect(isDeclared).toBe(true);
            });
        });
    });

    describe('TypeScript interface parity verification', () => {
        const expectedTypes = [
            'EngineAgent',
            'RoleBlueprint',
            'ModelConfig',
            'TokenUsage',
            'SkillManifest',
            'DangerLevel',
            'Permission'
        ];

        expectedTypes.forEach(typeName => {
            it(`should export type ${typeName} in generated.ts`, () => {
                const typeDeclarationRegex = new RegExp(`export\\s+type\\s+${typeName}\\b`);
                expect(generatedContent).toMatch(typeDeclarationRegex);
            });
        });
    });

    describe('Field-level type alignment checks', () => {
        it('should align RoleBlueprint fields', () => {
            // Locate the definition of RoleBlueprint in generated.ts
            const roleBlueprintMatch = generatedContent.match(/export\s+type\s+RoleBlueprint\s*=\s*\{([^}]+)\}/);
            expect(roleBlueprintMatch).not.toBeNull();
            
            const fieldsText = roleBlueprintMatch![1];
            
            // Check that required properties are present
            expect(fieldsText).toContain('id:');
            expect(fieldsText).toContain('name:');
            expect(fieldsText).toContain('department:');
            expect(fieldsText).toContain('description:');
            
            // Check optional parameters are present
            expect(fieldsText).toContain('skills?');
            expect(fieldsText).toContain('workflows?');
            expect(fieldsText).toContain('mcpTools?');
            expect(fieldsText).toContain('requiresOversight?');
            expect(fieldsText).toContain('modelId?');
        });

        it('should align ModelConfig fields', () => {
            const modelConfigMatch = generatedContent.match(/export\s+type\s+ModelConfig\s*=\s*\{([^}]+)\}/);
            expect(modelConfigMatch).not.toBeNull();
            
            const fieldsText = modelConfigMatch![1];
            expect(fieldsText).toContain('provider:');
            expect(fieldsText).toContain('modelId?');
            expect(fieldsText).toContain('apiKey?');
            expect(fieldsText).toContain('baseUrl?');
            expect(fieldsText).toContain('temperature:');
        });

        it('should align TokenUsage fields', () => {
            const tokenUsageMatch = generatedContent.match(/export\s+type\s+TokenUsage\s*=\s*\{([^}]+)\}/);
            expect(tokenUsageMatch).not.toBeNull();
            
            const fieldsText = tokenUsageMatch![1];
            expect(fieldsText).toContain('inputTokens?');
            expect(fieldsText).toContain('outputTokens?');
            expect(fieldsText).toContain('totalTokens?');
        });

        it('should align EngineAgent layout structure', () => {
            const lines = generatedContent.split('\n');
            const engineAgentLine = lines.find(line => line.includes('export type EngineAgent ='));
            expect(engineAgentLine).toBeDefined();
            
            // EngineAgent uses nested domain layers in TS
            expect(engineAgentLine).toContain('identity:');
            expect(engineAgentLine).toContain('models:');
            expect(engineAgentLine).toContain('economics:');
            expect(engineAgentLine).toContain('health:');
            expect(engineAgentLine).toContain('capabilities:');
            expect(engineAgentLine).toContain('state:');
            expect(engineAgentLine).toContain('requires_oversight:');
            expect(engineAgentLine).toContain('shadows_human_id:');
        });
    });
});
