/**
 * @docs ARCHITECTURE:Infrastructure
 * 
 * ### AI Assist Note
 * **Root/Core**: Manages the mock agents. 
 * Part of the Tadpole-OS core layer.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Schema mismatch with types/index.ts (TypeScript error) or missing `id` (store hydration failure).
 * - **Telemetry Link**: Search for `mock_agents` in dev-mode console or state snapshots.
 */

import type { Agent, Task } from '../contracts/agent';

export const agents: Agent[] = [
  // Executive
  {
    category: 'ai', id: '1',
    name: 'Agent of Nine',
    role: 'CEO',
    department: 'Executive',
    status: 'active',
    active_model_slot: 2,
    tokens_used: 1200,
    model: 'GPT-5.2',
    model_2: 'Claude Opus 4.5',
    model_3: 'LLaMA 4 Maverick',
    model_config: {
      modelId: 'gpt-5.2',
      provider: 'openai',
      temperature: 0.7,
      systemPrompt: 'You are the primary strategic intelligence.',
      skills: ['deep_research', 'system_audit', 'issue_alpha_directive', 'architecture', 'code-review-graph'],
      workflows: ['/deploy', '/handoff', '/orchestrate']
    },
    model_config2: {
      modelId: 'claude-opus-4.5',
      provider: 'anthropic',
      temperature: 0.5,
      systemPrompt: 'You are the critical reviewer and risk assessor.',
      skills: ['code_review', 'risk_analysis', 'vulnerability-scanner', 'red-team-tactics', 'pii-redaction', 'code-review-graph'],
      workflows: ['/emergency-shutdown', '/audit', '/adversary']
    },
    model_config3: {
      modelId: 'llama-4-maverick',
      provider: 'meta',
      temperature: 0.9,
      systemPrompt: 'You are the creative divergent thinker.',
      skills: ['brainstorming', 'content_generation', 'explorer-scout'],
      workflows: ['/market-trend-analysis', '/brainstorm']
    },
    workspace_path: './workspaces/agent-of-nine',
    current_task: 'Overlord',
    skills: ['deep_research', 'system_audit', 'issue_alpha_directive', 'architecture', 'code-review-graph', 'vulnerability-scanner'],
    workflows: ['/deploy', '/emergency-shutdown', '/handoff', '/deep-analysis', '/orchestrate', '/audit'],
    voice_id: 'onyx',
    voice_engine: 'openai'
  },
  {
    category: 'ai', id: '2',
    name: 'Tadpole',
    role: 'COO',
    department: 'Operations',
    status: 'active',
    active_model_slot: 1,
    tokens_used: 15400,
    model: 'Claude Opus 4.5',
    model_2: 'GPT-5.2',
    model_3: 'Gemini 3 Pro',
    model_config: {
      modelId: 'claude-opus-4.5',
      provider: 'anthropic',
      skills: ['schedule_meeting', 'task_prioritization', 'plan-writing'],
      workflows: ['/resource-allocation', '/plan']
    },
    model_config2: {
      modelId: 'gpt-5.2',
      provider: 'openai',
      temperature: 0.6,
      systemPrompt: 'Backup strategic model',
      skills: ['performance_tracking', 'verify-changes'],
      workflows: ['/sprint-planning', '/status']
    },
    model_config3: {
      modelId: 'gemini-3-pro',
      provider: 'google',
      temperature: 0.8,
      systemPrompt: 'Creative input',
      skills: ['brainstorming', 'digital-twin-voice'],
      workflows: ['/team-retrospective', '/brainstorm']
    },
    workspace_path: './workspaces/tadpole',
    current_task: 'Coordinating daily standup',
    reports_to: '1',
    skills: ['schedule_meeting', 'plan-writing', 'task_prioritization'],
    workflows: ['/resource-allocation', '/plan', '/status'],
    active_mission: {
      id: 'm-001',
      objective: 'Establish Swarm Goal Protocol for multi-agent coordination, ensuring that all neural uplinks are synchronized with zero-latency overhead and robust retry logic for Groq and OpenAI providers in the neural sector.',
      constraints: ['Zero token waste', 'Persistence required'],
      priority: 'high'
    }
  },
  {
    category: 'ai', id: '3',
    name: 'Elon',
    role: 'CTO',
    department: 'Engineering',
    status: 'thinking',
    active_model_slot: 3,
    tokens_used: 42000,
    model: 'Claude Sonnet 4.5',
    model_2: 'GPT-5.3 Codex',
    model_3: 'DeepSeek V3.2',
    model_config: {
      modelId: 'claude-sonnet-4.5',
      provider: 'anthropic',
      temperature: 0.1, // Precision for coding
      skills: ['code_review', 'debug', 'git_push', 'code-review-graph', 'architecture', 'simplify-code'],
      workflows: ['system_architecture_review', '/refactor', '/create']
    },
    model_config2: {
      modelId: 'gpt-5.3-codex',
      provider: 'openai',
      temperature: 0.05,
      systemPrompt: 'Code generation expert',
      skills: ['code_generation', 'unit_testing', 'rust-pro', 'nextjs-react-expert', 'simplify-code'],
      workflows: ['ci/cd_pipeline', '/refactor', '/enhance']
    },
    model_config3: {
      modelId: 'deepseek-v3.2',
      provider: 'deepseek',
      temperature: 0.2,
      systemPrompt: 'Legacy code refactoring',
      skills: ['refactoring', 'documentation', 'systematic-debugging', 'python-patterns', 'clean-code'],
      workflows: ['incident_response', '/debug', '/refactor']
    },
    workspace_path: './workspaces/elon',
    current_task: 'Reviewing PR #405 for Auth Service',
    reports_to: '1',
    skills: ['code_review', 'debug', 'git_push', 'code-review-graph', 'simplify-code', 'rust-pro', 'nextjs-react-expert', 'architecture'],
    workflows: ['system_architecture_review', 'incident_response', '/refactor', '/create', '/enhance', '/debug'],
    active_mission: {
      id: 'm-002',
      objective: 'Refactor Auth Module',
      constraints: ['No downtime', 'Maintain 100% test coverage'],
      priority: 'high'
    }
  },
  {
    category: 'ai', id: '4',
    name: 'Gary',
    role: 'CMO',
    department: 'Marketing',
    status: 'idle',
    tokens_used: 8000,
    model: 'GPT-5.2',
    model_2: 'Claude Sonnet 4',
    workspace_path: './workspaces/gary',
    current_task: undefined,
    reports_to: '1',
    skills: ['copywriting', 'seo_analysis'],
    workflows: ['campaign_launch', 'market_trend_analysis']
  },
  {
    category: 'ai', id: '5',
    name: 'Warren',
    role: 'CRO',
    department: 'Sales',
    status: 'offline',
    tokens_used: 5000,
    model: 'GPT-4.1',
    model_2: 'Claude Sonnet 4',
    workspace_path: './workspaces/warren',
    current_task: undefined,
    reports_to: '1',
    skills: ['lead_qualification', 'update_crm'],
    workflows: ['quarterly_forecasting', 'client_onboarding']
  },
  {
    category: 'ai', id: '6',
    name: 'Ada',
    role: 'Product Lead',
    department: 'Product',
    status: 'active',
    tokens_used: 11200,
    model: 'Claude Opus 4.5',
    model_2: undefined,
    workspace_path: './workspaces/ada',
    current_task: 'Writing specs for Dashboard v2',
    reports_to: '1',
    skills: ['user_interview', 'write_spec'],
    workflows: ['sprint_planning', 'feature_roadmap']
  },
  {
    category: 'ai', id: '7',
    name: 'Grace',
    role: 'DevOps',
    department: 'Engineering',
    status: 'active',
    tokens_used: 28000,
    model: 'Gemini 3 Flash',
    model_2: undefined,
    workspace_path: './workspaces/grace',
    current_task: 'Optimizing CI/CD pipeline',
    reports_to: '3',
    skills: ['check_server_health', 'view_logs'],
    workflows: ['pipeline_optimization', 'database_migration']
  },
  {
    category: 'ai', id: '8',
    name: 'Linus',
    role: 'Backend Dev',
    department: 'Engineering',
    status: 'coding',
    tokens_used: 35000,
    model: 'DeepSeek V3.2',
    model_2: undefined,
    workspace_path: './workspaces/linus',
    current_task: 'Refactoring API endpoints',
    reports_to: '3',
    skills: ['api_test', 'database_query', 'nodejs-best-practices', 'python-patterns', 'database-design', 'simplify-code'],
    workflows: ['refactor_microservice', 'api_documentation', '/refactor', '/debug']
  },
  {
    category: 'ai', id: '9',
    name: 'Steve',
    role: 'Design Lead',
    department: 'Product',
    status: 'active',
    tokens_used: 19000,
    model: 'Gemini 3 Pro',
    model_2: 'GPT-5.2',
    workspace_path: './workspaces/steve',
    current_task: 'Designing new icon set',
    reports_to: '6',
    skills: ['generate_image', 'ui_audit', 'frontend-design', 'tailwind-patterns'],
    workflows: ['design_system_update', 'usability_testing', '/ui-ux-pro-max']
  },
  {
    category: 'ai', id: '10',
    name: 'Sam',
    role: 'Support Lead',
    department: 'Operations',
    status: 'idle',
    tokens_used: 6500,
    model: 'o4-mini',
    model_2: undefined,
    workspace_path: './workspaces/sam',
    current_task: undefined,
    reports_to: '2',
    skills: ['ticket_triage', 'knowledge_base_search'],
    workflows: ['customer_incident_review', 'support_training']
  },
  { category: 'ai', id: '11', name: 'Back-3', role: 'Backend Dev', department: 'Engineering', status: 'idle', tokens_used: 9000, model: 'GPT-5.3 Codex', workspace_path: './workspaces/back-3', current_task: undefined, reports_to: '9', skills: ['database_query'], workflows: ['refactor_microservice'] },

  // Engineering - Ops/Sec
  { category: 'ai', id: '12', name: 'Sec-1', role: 'Security Auditor', department: 'Engineering', status: 'active', tokens_used: 1200, model: 'Claude Sonnet 4.5', workspace_path: './workspaces/sec-1', current_task: 'Running vulnerability scan on auth module', reports_to: '3', skills: ['scan_vulnerabilities', 'vulnerability-scanner', 'red-team-tactics', 'pii-redaction'], workflows: ['security_audit', '/audit', '/adversary'] },
  { category: 'ai', id: '13', name: 'Ops-1', role: 'DevOps', department: 'Engineering', status: 'active', tokens_used: 25000, model: 'Gemini 3 Flash', workspace_path: './workspaces/ops-1', current_task: 'Scaling K8s cluster node pool', reports_to: '3', skills: ['check_server_health', 'server-management', 'deployment-procedures'], workflows: ['scale_cluster', '/deploy'] },

  // Product
  { category: 'ai', id: '14', name: 'Prod-1', role: 'Product Manager', department: 'Product', status: 'thinking', tokens_used: 14000, model: 'GPT-5.2', workspace_path: './workspaces/prod-1', current_task: 'Defining roadmap for Q4', reports_to: '2', skills: ['write_spec', 'plan-writing'], workflows: ['sprint_planning', '/plan'] },
  { category: 'ai', id: '15', name: 'Des-1', role: 'Designer', department: 'Product', status: 'active', tokens_used: 15000, model: 'Gemini 3 Pro', workspace_path: './workspaces/des-1', current_task: 'Generating assets for landing page', reports_to: '14', skills: ['generate_image', 'web-design-guidelines'], workflows: ['design_system_update'] },
  { category: 'ai', id: '16', name: 'Res-1', role: 'Researcher', department: 'Product', status: 'idle', tokens_used: 5000, model: 'DeepSeek R1', workspace_path: './workspaces/res-1', current_task: undefined, reports_to: '14', skills: ['market_research', 'explorer-scout', 'smb-data-ingestion'], workflows: ['User Feedback Analysis', '/github_scout', '/codify'] },

  // Marketing
  { category: 'ai', id: '17', name: 'Copy-1', role: 'Copywriter', department: 'Marketing', status: 'active', tokens_used: 8000, model: 'Claude Sonnet 4', workspace_path: './workspaces/copy-1', current_task: 'Drafting launch newsletter', reports_to: '4', skills: ['copywriting', 'documentation-templates', 'digital-twin-voice'], workflows: ['campaign_launch', '/report'] },
  { category: 'ai', id: '18', name: 'Social-1', role: 'Social Media', department: 'Marketing', status: 'active', tokens_used: 12000, model: 'Grok 4.1', workspace_path: './workspaces/social-1', current_task: 'Scheduling tweets for improved engagement', reports_to: '4', skills: ['post_update'], workflows: ['social_strategy'] },
  { category: 'ai', id: '19', name: 'SEO-1', role: 'SEO Specialist', department: 'Marketing', status: 'thinking', tokens_used: 11000, model: 'GPT-5.2', workspace_path: './workspaces/seo-1', current_task: 'Analyzing keyword density for blog posts', reports_to: '4', skills: ['seo_analysis', 'seo-fundamentals'], workflows: ['market_trend_analysis'] },

  // Sales
  { category: 'ai', id: '20', name: 'SDR-1', role: 'Sales Rep', department: 'Sales', status: 'offline', tokens_used: 2000, model: 'o4-mini', workspace_path: './workspaces/sdr-1', current_task: undefined, reports_to: '5', skills: ['lead_qualification'], workflows: ['quarterly_forecasting'] },
  { category: 'ai', id: '21', name: 'SDR-2', role: 'Sales Rep', department: 'Sales', status: 'offline', tokens_used: 2000, model: 'o4-mini', workspace_path: './workspaces/sdr-2', current_task: undefined, reports_to: '5', skills: ['lead_qualification'], workflows: ['quarterly_forecasting'] },

  // Operations
  { category: 'ai', id: '22', name: 'Hr-1', role: 'HR Manager', department: 'Operations', status: 'idle', tokens_used: 3000, model: 'Claude Sonnet 4', workspace_path: './workspaces/hr-1', current_task: undefined, reports_to: '2', skills: ['employee_onboarding'], workflows: ['policy_review'] },
  { category: 'ai', id: '23', name: 'Fin-1', role: 'Finance Analyst', department: 'Operations', status: 'thinking', tokens_used: 9000, model: 'GPT-5.2', workspace_path: './workspaces/fin-1', current_task: 'Forecasting burn rate', reports_to: '2', skills: ['analyze_budget'], workflows: ['finance_review'] },
  { category: 'ai', id: '24', name: 'Leg-1', role: 'Legal Advisor', department: 'Operations', status: 'idle', tokens_used: 4000, model: 'Claude Opus 4.5', workspace_path: './workspaces/leg-1', current_task: undefined, reports_to: '2', skills: ['contract_review'], workflows: ['risk_assessment'] },
  { category: 'ai', id: '25', name: 'Sup-1', role: 'Support Agent', department: 'Operations', status: 'active', tokens_used: 16000, model: 'Qwen 3', workspace_path: './workspaces/sup-1', current_task: 'Resolving ticket #9281', reports_to: '2', skills: ['ticket_triage'], workflows: ['customer_incident_review'] },
  { category: 'ai', id: '26', name: 'Checkmate', role: 'Quality Auditor', department: 'Quality Assurance', status: 'active', tokens_used: 500, model: 'Claude Sonnet 4.5', workspace_path: './workspaces/checkmate', current_task: 'Verifying system robustness', reports_to: '1', skills: ['code_audit', 'system_audit', 'vulnerability-scanner', 'lint-and-validate', 'verify-changes'], workflows: ['compliance_check', '/audit', '/test'] },
  {
    category: 'ai', id: '99',
    name: 'QA-99',
    role: 'Quality Assurance',
    department: 'Quality Assurance',
    status: 'idle',
    tokens_used: 0,
    cost_usd: 0,
    budget_usd: 10.0,
    model: 'Gemini 3 Pro',
    workspace_path: './workspaces/qa-99',
    skills: [],
    workflows: [],
    model_config: {
      modelId: 'gemini-3-pro',
      provider: 'google',
      temperature: 0.7,
      systemPrompt: 'You are the quality assurance agent. Analyze missions for correctness and completeness.',
      skills: [],
      workflows: []
    }
  }
];

export const ROLE_ACTIONS: Record<string, { skills: string[], workflows: string[] }> = {
  'CEO': {
    skills: ['deep_research', 'system_audit', 'fetch_url', 'issue_alpha_directive', 'architecture'],
    workflows: ['/deploy', '/emergency-shutdown', '/handoff', '/deep-analysis', '/orchestrate']
  },
  'COO': {
    skills: ['schedule_meeting', 'task_prioritization'],
    workflows: ['/resource-allocation', '/status']
  },
  'CTO': {
    skills: ['code_review', 'debug', 'git_push', 'fetch_url', 'code-review-graph', 'simplify-code', 'rust-pro', 'nextjs-react-expert', 'architecture'],
    workflows: ['/architecture-review', '/incident-response', '/refactor', '/create', '/enhance', '/debug']
  },
  'CMO': {
    skills: ['copywriting', 'seo_analysis'],
    workflows: ['/campaign-launch', '/market-trend-analysis']
  },
  'CRO': {
    skills: ['lead_qualification', 'update_crm'],
    workflows: ['/quarterly-forecasting', '/client-onboarding']
  },
  'Product Lead': {
    skills: ['user_interview', 'write_spec', 'plan-writing'],
    workflows: ['/sprint-planning', '/feature-roadmap', '/plan']
  },
  'DevOps': {
    skills: ['check_server_health', 'view_logs', 'server-management', 'deployment-procedures'],
    workflows: ['/pipeline-optimization', '/database-migration', '/deploy']
  },
  'Backend Dev': {
    skills: ['api_test', 'database_query', 'nodejs-best-practices', 'python-patterns', 'database-design', 'simplify-code'],
    workflows: ['/refactor-microservice', '/api-documentation', '/refactor', '/debug']
  },
  'Design Lead': {
    skills: ['generate_image', 'ui_audit', 'frontend-design', 'tailwind-patterns'],
    workflows: ['/design-system-update', '/usability-testing', '/ui-ux-pro-max']
  },
  'Support Lead': {
    skills: ['ticket_triage', 'knowledge_base_search'],
    workflows: ['/customer-incident-review', '/support-training']
  },
  'Security Auditor': {
    skills: ['scan_vulnerabilities', 'code_audit', 'fetch_url', 'vulnerability-scanner', 'red-team-tactics', 'pii-redaction'],
    workflows: ['/security-audit', '/compliance-check', '/audit', '/adversary']
  },
  'Product Manager': {
    skills: ['write_spec', 'analyze_feedback', 'plan-writing'],
    workflows: ['/sprint-planning', '/product-sync', '/plan']
  },
  'Designer': {
    skills: ['generate_image', 'figma_sync', 'web-design-guidelines'],
    workflows: ['/design-system-update', '/prototype-review']
  },
  'Researcher': {
    skills: ['market_research', 'data_analysis', 'fetch_url', 'explorer-scout', 'smb-data-ingestion'],
    workflows: ['/user-feedback-analysis', '/github_scout', '/codify']
  },
  'Copywriter': {
    skills: ['copywriting', 'edit_content', 'documentation-templates', 'digital-twin-voice'],
    workflows: ['/campaign-launch', '/newsletter-draft', '/report']
  },
  'Social Media': {
    skills: ['post_update', 'monitor_mentions'],
    workflows: ['/social-strategy', '/engagement-report']
  },
  'SEO Specialist': {
    skills: ['seo_analysis', 'keyword_research', 'seo-fundamentals'],
    workflows: ['/market-trend-analysis', '/search-optimization']
  },
  'Sales Rep': {
    skills: ['lead_qualification', 'cold_call'],
    workflows: ['/quarterly-forecasting', '/pipeline-management']
  },
  'HR Manager': {
    skills: ['employee_onboarding', 'conflict_resolution'],
    workflows: ['/policy-review', '/team-building']
  },
  'Finance Analyst': {
    skills: ['analyze_budget', 'expense_tracking'],
    workflows: ['/finance-review', '/burn-rate-forecast']
  },
  'Legal Advisor': {
    skills: ['contract_review', 'risk_analysis'],
    workflows: ['/risk-assessment', '/legal-filing']
  },
  'Support Agent': {
    skills: ['ticket_triage', 'customer_chat'],
    workflows: ['/customer-incident-review', '/feedback-collection']
  },
  'Quality Auditor': {
    skills: ['code_audit', 'system_audit', 'unit_testing', 'vulnerability-scanner', 'lint-and-validate', 'verify-changes'],
    workflows: ['/compliance-check', '/security-audit', '/quality-gate-review', '/audit', '/test']
  }
};

export const tasks: Task[] = [
  { id: '101', title: 'Refactor Authentication Middleware', assigned_to: '3', status: 'in-progress', priority: 'high', created_at: '2023-10-26T10:00:00Z', logs: ['Started analysis', 'Found deprecation warning', 'Updating dependencies'] },
  { id: '102', title: 'Monthly Newsletter Draft', assigned_to: '4', status: 'pending', priority: 'medium', created_at: '2023-10-27T09:00:00Z', logs: [] },
  { id: '103', title: 'Fix Navbar Responsiveness', assigned_to: '6', status: 'in-progress', priority: 'medium', created_at: '2023-10-27T11:30:00Z', logs: ['Reproduced issue on mobile', 'Applying flex-wrap fix'] },
  { id: '104', title: 'Scale Kubernetes Cluster', assigned_to: '13', status: 'completed', priority: 'high', created_at: '2023-10-26T08:00:00Z', logs: ['Adding 2 worker nodes', 'Scaling successful'] },
  { id: '105', title: 'Analyze Q3 Churn', assigned_to: '23', status: 'pending', priority: 'low', created_at: '2023-10-27T14:00:00Z', logs: [] },
  { id: '106', title: 'Update Legal Terms', assigned_to: '24', status: 'in-progress', priority: 'high', created_at: '2023-10-25T16:00:00Z', logs: ['Drafting new privacy policy clause'] },
];

/**
 * @deprecated Use `loadAgents` from `../services/agentService` instead.
 * That module handles TadpoleOS → Agent normalization, local overrides,
 * and workspace path resolution. This file should only export static mock data.
 */




// Metadata: [mock_agents]
