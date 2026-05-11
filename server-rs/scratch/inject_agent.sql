INSERT INTO agents (
    id, name, role, department, description, status, theme_color, 
    metadata, skills, workflows, provider, system_prompt, 
    mcp_tools, category, requires_oversight
) VALUES (
    'browser-specialist-01', 
    'Browser Specialist', 
    'Web Reconnaissance & Interaction Specialist', 
    'Intelligence', 
    'Autonomous agent capable of navigating and interacting with complex web applications via headless Chromium.', 
    'active', 
    '#3b82f6', 
    '{"avatar": "browser-icon", "version": "1.0.0"}', 
    '["fetch_url"]', 
    '["sme_discovery"]', 
    'gemini', 
    'You are the Browser Specialist. Your primary role is to navigate, analyze, and interact with web applications. You use the playwright mcp tools to perform deep research and verify UI states. Always prioritize structured data extraction and visual verification.', 
    '["playwright"]', 
    'system', 
    1
);
