/**
 * @docs ARCHITECTURE:Contracts
 * 
 * ### AI Assist Note
 * **Skill & Workflow Registry**: Authoritative strongly-typed contracts mirroring
 * production system capabilities in `.agent/skills/` and `.agent/workflows/`.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Schema mismatch or unknown skill string passed to execution engine.
 * - **Telemetry Link**: Search `[SkillRegistry]` in UI & execution logs.
 */

export const SYSTEM_SKILLS = [
  'aletheia-reasoning',
  'analyze_budget',
  'analyze_feedback',
  'api-patterns',
  'api_test',
  'app-builder',
  'architecture',
  'bash-linux',
  'batch-operations',
  'behavioral-modes',
  'brainstorming',
  'check_server_health',
  'clean-code',
  'code-review',
  'code-review-checklist',
  'code-review-graph',
  'code_audit',
  'code_generation',
  'code_review',
  'cold_call',
  'conflict_resolution',
  'content_generation',
  'context-compression',
  'contract_review',
  'coordinator-mode',
  'copywriting',
  'customer_chat',
  'data_analysis',
  'database-design',
  'database_query',
  'debug',
  'deep_research',
  'deployment-procedures',
  'digital-twin-voice',
  'documentation',
  'documentation-templates',
  'domain-modeling',
  'edit_content',
  'employee_onboarding',
  'expense_tracking',
  'explorer-scout',
  'fetch_url',
  'figma_sync',
  'frontend-design',
  'game-development',
  'gemma4-local',
  'generate_image',
  'geo-fundamentals',
  'git_push',
  'handoff',
  'i18n-localization',
  'improve-codebase-architecture',
  'intelligent-routing',
  'issue_alpha_directive',
  'keyword_research',
  'knowledge_base_search',
  'lead_qualification',
  'lint-and-validate',
  'market_research',
  'mcp-builder',
  'memory-system',
  'mission-analyst',
  'mobile-design-android',
  'mobile-design-ios',
  'monitor_mentions',
  'nextjs-react-expert',
  'nodejs-best-practices',
  'parallel-agents',
  'performance-profiling',
  'performance_tracking',
  'pii-redaction',
  'plan-writing',
  'post_update',
  'powershell-windows',
  'python-patterns',
  'red-team-tactics',
  'refactoring',
  'research',
  'risk_analysis',
  'rust-pro',
  'scan_vulnerabilities',
  'schedule_meeting',
  'seo-fundamentals',
  'seo_analysis',
  'server-management',
  'simplify-code',
  'skillify',
  'smb-data-ingestion',
  'system_audit',
  'systematic-debugging',
  'tailwind-patterns',
  'task_prioritization',
  'tdd-workflow',
  'testing-patterns',
  'ticket_triage',
  'to-spec',
  'ui_audit',
  'unit_testing',
  'update_crm',
  'user_interview',
  'verify-changes',
  'view_logs',
  'vulnerability-scanner',
  'wayfinder',
  'web-design-guidelines',
  'webapp-testing',
  'world-model-synthesis',
  'write_spec'
] as const;

export type SystemSkill = typeof SYSTEM_SKILLS[number];

export const SYSTEM_WORKFLOWS = [
  '/adversary',
  '/api-documentation',
  '/architecture-review',
  '/anneal',
  '/audit',
  '/brainstorm',
  '/burn-rate-forecast',
  '/campaign-launch',
  '/ci-cd-pipeline',
  '/client-onboarding',
  '/codify',
  '/compliance-check',
  '/create',
  '/customer-incident-review',
  '/debug',
  '/deep-analysis',
  '/deploy',
  '/design-system-update',
  '/emergency-shutdown',
  '/engagement-report',
  '/enhance',
  '/feature-roadmap',
  '/feedback-collection',
  '/finance-review',
  '/github_scout',
  '/handoff',
  '/incident-response',
  '/legal-filing',
  '/market-trend-analysis',
  '/migrate',
  '/newsletter-draft',
  '/onboard',
  '/orchestrate',
  '/pipeline-management',
  '/pipeline-optimization',
  '/plan',
  '/policy-review',
  '/preview',
  '/product-sync',
  '/prototype-review',
  '/quality-gate-review',
  '/quarterly-forecasting',
  '/refactor',
  '/refactor-microservice',
  '/report',
  '/resource-allocation',
  '/risk-assessment',
  '/scale-cluster',
  '/search-optimization',
  '/security-audit',
  '/social-strategy',
  '/sprint-planning',
  '/status',
  '/support-training',
  '/team-building',
  '/team-retrospective',
  '/test',
  '/ui-ux-pro-max',
  '/usability-testing',
  '/user-feedback-analysis'
] as const;

export type SystemWorkflow = typeof SYSTEM_WORKFLOWS[number];

// High-performance O(1) Sets for Type Predicate validation [SkillRegistry]
const SKILL_SET = new Set<string>(SYSTEM_SKILLS);
const WORKFLOW_SET = new Set<string>(SYSTEM_WORKFLOWS);

/**
 * Checks if a given skill string matches a known production SystemSkill identifier in O(1) time.
 */
export function is_valid_skill(skill: string): skill is SystemSkill {
  return SKILL_SET.has(skill);
}

/**
 * Checks if a given workflow string matches a known production SystemWorkflow slash command in O(1) time.
 */
export function is_valid_workflow(workflow: string): workflow is SystemWorkflow {
  return WORKFLOW_SET.has(workflow);
}

/**
 * Returns a sorted copy of all available system skill identifiers.
 */
export function get_all_system_skills(): SystemSkill[] {
  return [...SYSTEM_SKILLS].sort();
}

/**
 * Returns a sorted copy of all available system workflow slash commands.
 */
export function get_all_system_workflows(): SystemWorkflow[] {
  return [...SYSTEM_WORKFLOWS].sort();
}
