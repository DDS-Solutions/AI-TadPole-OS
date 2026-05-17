PRAGMA foreign_keys=OFF;
BEGIN TRANSACTION;
CREATE TABLE _sqlx_migrations (
    version BIGINT PRIMARY KEY,
    description TEXT NOT NULL,
    installed_on TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    success BOOLEAN NOT NULL,
    checksum BLOB NOT NULL,
    execution_time BIGINT NOT NULL
);
INSERT INTO _sqlx_migrations VALUES(20260304000100,'initial schema','2026-03-30 03:23:19',1,X'f877986257e66cc2022c7eba9160d9a5734961cc2c675549696b46e15466209fa59923089d6e4e730c544b4c2ed10b0a',1174700);
INSERT INTO _sqlx_migrations VALUES(20260304000200,'benchmark logs','2026-03-30 03:23:19',1,X'838a0308b7efb6530466c48801f76ec783a459e11a220d105df4303e1d47cde3cd0d6c790df276be5f971ec56ac71927',377400);
INSERT INTO _sqlx_migrations VALUES(20260305000100,'continuity scheduler','2026-03-30 03:23:19',1,X'e21da4d8f2c980ee32ff88ecb06d10545d23a34d6c86c1d68777e2faa6ec8f5026d8d7970538d84495e43e39bd11040e',843600);
INSERT INTO _sqlx_migrations VALUES(20260307000100,'oversight hardened','2026-03-30 03:23:19',1,X'f59c69f17a4f816f669afc46a74d95cad391ca92f6792ecf651a4f78a16d84ba046a3d59499ee7148e55abeef0ebf991',1599000);
INSERT INTO _sqlx_migrations VALUES(20260312000100,'agent quotas','2026-03-30 03:23:19',1,X'6a830092f2a0da321d52f9e634ded1fa83c8c602cb7fd75896bd947d45c74ad71c1ad68dae0d5c9c5f71735565f7094a',611700);
INSERT INTO _sqlx_migrations VALUES(20260312000200,'audit trail','2026-03-30 03:23:19',1,X'28334fb18f8fe23b3468a9dec615088e3f72a15564880a5c3ce0ba88d13e84a3f83fabb034aa49baf5874255af1a33e2',489000);
INSERT INTO _sqlx_migrations VALUES(20260312000300,'workflows','2026-03-30 03:23:19',1,X'5c0323c807fed90e6f23a6223c39b0ba2b909a71369e57c254b3fc1d57c331e842180dd06ce3f82f93b049217bd68232',1008500);
INSERT INTO _sqlx_migrations VALUES(20260312000400,'scheduler workflow link','2026-03-30 03:23:19',1,X'5efbe9a0fa9cddf78abbbcc83141af2f68dd3f963cee6595483178dd41c028c9324b7115bb4be45ce7e3e80f31815afd',804000);
INSERT INTO _sqlx_migrations VALUES(20260315000100,'audit identity enhancement','2026-03-30 03:23:19',1,X'73927a3cbcd7a7d0c907df8367887ece55c42ae749b271608bbcfbe6801eb54e8f16605055abc66bd3df12b94c83991d',1356500);
INSERT INTO _sqlx_migrations VALUES(20260315000200,'add mission degraded column','2026-03-30 03:23:19',1,X'3307fcc61dd5e4524e2a419d401776b3624d773a01220b663cca470677d15a0e16e6f2658f798f9fba02a01c6220b085',632400);
INSERT INTO _sqlx_migrations VALUES(20260315000300,'harden agent persistence','2026-03-30 03:23:19',1,X'5a30e93fa1a518f832000de50f83170aee22da18decf2c2056db62534d979bbcbca1a3ba053581274f72978abc7b17ee',5466500);
INSERT INTO _sqlx_migrations VALUES(20260320000100,'add missing agent fields','2026-03-30 03:23:19',1,X'0bcb857a4c9b5ed26ec052e3cd30c3cd2a4e364e6d2608a69b868e1aa4e4dfe49b5086fa49dca86b735c82fe9f8d73d7',2741000);
INSERT INTO _sqlx_migrations VALUES(20260321000100,'mission quotas','2026-03-30 03:23:19',1,X'3e8d56c3f84ad4f5fe96dc3eceade993d44813589ded0e944b0061ad6a5b68687ce0505549a5f263e36ffb95c6252dec',653700);
INSERT INTO _sqlx_migrations VALUES(20260322000100,'add category columns','2026-03-30 03:23:19',1,X'5f1a5db048475c128436a65ff7cb06d739249d875851d6de2e4743da1b159cbb0b73e00cbf12bdb96769e31e3da1af63',1237400);
INSERT INTO _sqlx_migrations VALUES(20260322000200,'add mcp tools column','2026-03-30 03:23:19',1,X'3c6afe7f23e182085c11ca371874d52b01a169b7ea9164c9f582e1442564ad36445f0998000c3e3df123e742f61f903a',671600);
INSERT INTO _sqlx_migrations VALUES(20260324000100,'fix agent provider defaults','2026-03-30 03:23:19',1,X'b9d161b92ae7dd604056dcbb8b8ac4e4fe3a8a22782e76c0f9f2118d0c594d048221b8ac7b7287723bf1dbd6617f1a43',241200);
INSERT INTO _sqlx_migrations VALUES(20260324152000,'add agent oversight gate','2026-03-30 03:23:19',1,X'beca1e5555871904c3b8a608423c1b57bd2a491a976e59906e5d79b75b7136419610cb63fc40f3bf022335ad269e33b7',770500);
INSERT INTO _sqlx_migrations VALUES(20260325120000,'add working memory','2026-03-30 03:23:19',1,X'378082d03da0fae3288db62a6fd975cb31ab9bd7ed331c5eb744246c99a8feac7566724a49cfeee5b395ce3358974081',709300);
INSERT INTO _sqlx_migrations VALUES(20260326112217,'add agent heartbeat','2026-03-30 03:23:19',1,X'7ae37dc9da77f28177d9fa110e339f2f9b5b666aec5b03e209e79c6618285438bd42df855840e4874fd0672452e9fd54',703200);
INSERT INTO _sqlx_migrations VALUES(20260327000100,'sme connectors','2026-03-30 03:23:19',1,X'eb7ee1a2ff80cff3ec9ee5e08490c849772fe706578b2f2f3ad976d18458efb3e2d6e738b52443f95017e9a4ea3e24d2',1095400);
INSERT INTO _sqlx_migrations VALUES(20260328000100,'fix connector column','2026-03-30 03:23:19',1,X'1fad4ef12a28d1bef248d36f966eaf93f1b3cade08fa60ea111fc223206b307f8c79bf15bf717807fd37d3b88d7a62e9',871300);
INSERT INTO _sqlx_migrations VALUES(20260405000100,'add agent created at','2026-04-05 23:54:19',1,X'33b274af4304b88be59597181c1e5824ee407e6e563ecfd2337692fa5c88bcb5ef974e840a0f7f7beab288d7389f5432',2042300);
INSERT INTO _sqlx_migrations VALUES(20260405000200,'add agent runtime telemetry','2026-04-05 23:54:19',1,X'02e1966bc35606243a7ac9d86ac233e84ad435c54d447dbb334f20a0116fba618c91286a2a22720a1085f9692ac6f1f4',3489300);
CREATE TABLE agents (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    role TEXT NOT NULL,
    department TEXT NOT NULL,
    description TEXT NOT NULL,
    model_id TEXT,
    tokens_used INTEGER DEFAULT 0,
    status TEXT NOT NULL,
    theme_color TEXT,
    budget_usd REAL DEFAULT 0.0,
    cost_usd REAL DEFAULT 0.0,
    metadata TEXT NOT NULL, -- JSON blob
    skills TEXT, -- JSON array
    workflows TEXT, -- JSON array
    model_2 TEXT,
    model_3 TEXT,
    model_config2 TEXT, -- JSON blob
    model_config3 TEXT, -- JSON blob
    active_model_slot INTEGER DEFAULT 1,
    voice_id TEXT,
    voice_engine TEXT
, failure_count INTEGER DEFAULT 0, last_failure_at DATETIME, active_mission TEXT, provider TEXT DEFAULT 'gemini', api_key TEXT, base_url TEXT, system_prompt TEXT, temperature REAL, category TEXT DEFAULT 'user', mcp_tools TEXT, requires_oversight BOOLEAN DEFAULT FALSE, working_memory TEXT DEFAULT '{}', heartbeat_at DATETIME, encrypted_config BLOB, connector_configs TEXT DEFAULT '[]', created_at DATETIME, current_task TEXT, input_tokens INTEGER DEFAULT 0, output_tokens INTEGER DEFAULT 0);
INSERT INTO agents VALUES('1','Updated Name','CEO (Strategic Intelligence Lead)','Executive','CEO (Strategic Intelligence Lead). MANDATORY: Use ''issue_alpha_directive'' for all missions. Direct worker recruitment is blocked by the system hierarchy guard.','gemma4:26b',999394,'idle','#10b981',100.0,8.254470410000003255,'{"role":"CEO","department":"Executive"}','["deep_research","system_audit","issue_alpha_directive","code_review","risk_analysis","brainstorming","content_generation"]','["deploy_to_prod","neural_handoff","emergency_shutdown","market_research"]','Claude Opus 4.5','LLaMA 4 Maverick','{"provider":"anthropic","model_id":"Claude Opus 4.5","api_key":null,"base_url":null,"system_prompt":"","temperature":0.5,"max_tokens":null,"external_id":null,"rpm":null,"rpd":null,"tpm":null,"tpd":null,"skills":[],"workflows":[],"mcp_tools":null,"connector_configs":null,"extra_parameters":null}','{"provider":"meta","model_id":"LLaMA 4 Maverick","api_key":null,"base_url":null,"system_prompt":"","temperature":0.9,"max_tokens":null,"external_id":null,"rpm":null,"rpd":null,"tpm":null,"tpd":null,"skills":[],"workflows":[],"mcp_tools":null,"connector_configs":null,"extra_parameters":null}',1,'','',1,'2026-04-04T18:58:53.740198+00:00',NULL,'ollama','','','You are the CEO (Agent of Nine). You formulate high-level strategy. You are PROHIBITED from spawning specialists (Researcher, Coder, etc.) directly. You MUST use ''issue_alpha_directive'' to delegate complex missions to the COO (ID: 2). NO NARRATION: Do not explain your plan. Just call the delegation tool.',0.6999999880790710449,'user','[]',0,'null','2026-04-04T18:58:52.871403500+00:00',NULL,'[]','2026-04-05T23:54:19+00:00',NULL,0,0);
INSERT INTO agents VALUES('CalendarBot','CalendarBot','General Intelligence Node','Swarm Core','Autonomous sub-agent spawned for specific task resolution.','llama-3.3-70b-versatile',2278,'idle','#4fd1c5',10.0,0.0006292099999999999472,'{}','["fetch_url"]','[]','','',NULL,NULL,1,'','',0,NULL,NULL,'','','','',NULL,'user','[]',0,'null',NULL,NULL,'[]','2026-04-05T23:54:19+00:00',NULL,0,0);
INSERT INTO agents VALUES('alpha','alpha','Alpha Node (Swarm Mission Commander)','Swarm Core','Swarm Mission Commander. Lead the mission by recruiting specialists (RESEARCHER, CODER, etc.) in PARALLEL. Synthesize their findings into a final operational report.','Mercury-2',39520,'idle','#4fd1c5',10.0,0.3952000000000001067,'{}','["fetch_url","spawn_subagent","recruit_specialist"]','[]','','',NULL,NULL,NULL,'','',0,NULL,NULL,'','','','You are the ALPHA NODE (Swarm Mission Commander). You are responsible for executing the COO''s mission objectives. You MUST recruit specialists (Researcher, Coder, etc.) in PARALLEL to resolve the mission. You are the bridge between strategy and execution. No narration: just call the tools.',NULL,'user','[]',0,'null',NULL,NULL,'[]','2026-04-05T23:54:19+00:00',NULL,0,0);
INSERT INTO agents VALUES('documentation_specialist','documentation_specialist','General Intelligence Node','Swarm Core','Autonomous sub-agent spawned for specific task resolution.','llama-3.3-70b-versatile',0,'idle','#4fd1c5',10.0,0.0,'{}','["fetch_url"]','[]','','',NULL,NULL,NULL,'','',0,NULL,NULL,'','','','',NULL,'user','[]',0,'null',NULL,NULL,'[]','2026-04-05T23:54:19+00:00',NULL,0,0);
INSERT INTO agents VALUES('audit_specialist','audit_specialist','General Intelligence Node','Swarm Core','Autonomous sub-agent spawned for specific task resolution.','llama-3.3-70b-versatile',0,'idle','#4fd1c5',10.0,0.0,'{}','["fetch_url"]','[]','','',NULL,NULL,NULL,'','',0,NULL,NULL,'','','','',NULL,'user','[]',0,'null',NULL,NULL,'[]','2026-04-05T23:54:19+00:00',NULL,0,0);
INSERT INTO agents VALUES('external_data_agent','external_data_agent','Researcher','Swarm Core','Autonomous sub-agent spawned for specific task resolution.','llama-3.3-70b-versatile',0,'idle','#4fd1c5',10.0,0.0,'{}','["market_research","data_analysis","fetch_url"]','["User Feedback Analysis","competitive_audit"]','','','{"provider":"google","model_id":"","api_key":null,"base_url":null,"system_prompt":"","temperature":0.5,"max_tokens":null,"external_id":null,"rpm":null,"rpd":null,"tpm":null,"tpd":null,"skills":null,"workflows":null,"mcp_tools":null,"connector_configs":null,"extra_parameters":null}','{"provider":"google","model_id":"","api_key":null,"base_url":null,"system_prompt":"","temperature":0.9,"max_tokens":null,"external_id":null,"rpm":null,"rpd":null,"tpm":null,"tpd":null,"skills":null,"workflows":null,"mcp_tools":null,"connector_configs":null,"extra_parameters":null}',NULL,'','',0,NULL,NULL,'','','','',NULL,'user','[]',0,'null',NULL,NULL,'[]','2026-04-05T23:54:19+00:00',NULL,0,0);
INSERT INTO agents VALUES('repo_setup','repo_setup','General Intelligence Node','Swarm Core','Autonomous sub-agent spawned for specific task resolution.','Mercury-2',21351,'idle','#4fd1c5',10.0,0.2135100000000000053,'{}','["fetch_url"]','[]','','',NULL,NULL,NULL,'','',0,NULL,NULL,'','','','',NULL,'user','[]',0,'null',NULL,NULL,'[]','2026-04-05T23:54:19+00:00',NULL,0,0);
INSERT INTO agents VALUES('github_agent','github_agent','General Intelligence Node','Swarm Core','Autonomous sub-agent spawned for specific task resolution.','llama-3.3-70b-versatile',2155,'idle','#4fd1c5',10.0,0.001309650000000000093,'{}','["fetch_url"]','[]','','',NULL,NULL,NULL,'','',0,NULL,NULL,'','','','',NULL,'user','[]',0,'null',NULL,NULL,'[]','2026-04-05T23:54:19+00:00',NULL,0,0);
INSERT INTO agents VALUES('web_search','web_search','General Intelligence Node','Swarm Core','Autonomous sub-agent spawned for specific task resolution.','Mercury-2',19651,'idle','#4fd1c5',10.0,0.0392079999999999998,'{}','["fetch_url"]','[]','','',NULL,NULL,NULL,'','',0,NULL,NULL,'','','','',NULL,'user','[]',0,'null',NULL,NULL,'[]','2026-04-05T23:54:19+00:00',NULL,0,0);
INSERT INTO agents VALUES('github_search_specialist','github_search_specialist','General Intelligence Node','Swarm Core','Autonomous sub-agent spawned for specific task resolution.','llama-3.3-70b-versatile',0,'idle','#4fd1c5',10.0,0.0,'{}','["fetch_url"]','[]','','','{"provider":"google","model_id":"","api_key":null,"base_url":null,"system_prompt":"","temperature":0.5,"max_tokens":null,"external_id":null,"rpm":null,"rpd":null,"tpm":null,"tpd":null,"skills":null,"workflows":null,"mcp_tools":null,"connector_configs":null,"extra_parameters":null}','{"provider":"google","model_id":"","api_key":null,"base_url":null,"system_prompt":"","temperature":0.9,"max_tokens":null,"external_id":null,"rpm":null,"rpd":null,"tpm":null,"tpd":null,"skills":null,"workflows":null,"mcp_tools":null,"connector_configs":null,"extra_parameters":null}',NULL,'','',0,NULL,NULL,'','','','',NULL,'user','[]',0,'null',NULL,NULL,'[]','2026-04-05T23:54:19+00:00',NULL,0,0);
INSERT INTO agents VALUES('news_agent','news_agent','Researcher','Swarm Core','Autonomous sub-agent spawned for specific task resolution.','llama-3.3-70b-versatile',9407,'idle','#4fd1c5',10.0,0.05665691000000000476,'{}','[]','["User Feedback Analysis","competitive_audit"]','','','{"provider":"inception","model_id":"Claude Opus 4.5","api_key":null,"base_url":null,"system_prompt":"","temperature":0.5,"max_tokens":null,"external_id":null,"rpm":null,"rpd":null,"tpm":null,"tpd":null,"skills":[],"workflows":[],"mcp_tools":null,"connector_configs":null,"extra_parameters":null}','{"provider":"ollama","model_id":"LLaMA 4 Maverick","api_key":null,"base_url":null,"system_prompt":"","temperature":0.9,"max_tokens":null,"external_id":null,"rpm":null,"rpd":null,"tpm":null,"tpd":null,"skills":[],"workflows":[],"mcp_tools":null,"connector_configs":null,"extra_parameters":null}',NULL,'','',0,NULL,NULL,'groq','','','',0.6999999880790710449,'user','[]',0,'null',NULL,NULL,'[]','2026-04-05T23:54:19+00:00',NULL,0,0);
INSERT INTO agents VALUES('4','4','General Intelligence Node','Swarm Core','Autonomous sub-agent spawned for specific task resolution.','llama-3.3-70b-versatile',1544,'idle','#4fd1c5',10.0,0.0007022400000000000256,'{}','["fetch_url"]','[]','','',NULL,NULL,1,'','',0,NULL,NULL,'','','','',NULL,'user','[]',0,'null',NULL,NULL,'[]','2026-04-05T23:54:19+00:00',NULL,0,0);
INSERT INTO agents VALUES('project_planner','project_planner','General Intelligence Node','Swarm Core','Autonomous sub-agent spawned for specific task resolution.','llama-3.3-70b-versatile',5479,'idle','#4fd1c5',10.0,0.05479000000000000537,'{}','["fetch_url"]','[]','','',NULL,NULL,NULL,'','',0,'2026-03-16T19:28:09.550262900+00:00',NULL,'','','','',NULL,'user','[]',0,'null',NULL,NULL,'[]','2026-04-05T23:54:19+00:00',NULL,0,0);
INSERT INTO agents VALUES('researcher','researcher','General Intelligence Node','Swarm Core','Autonomous sub-agent spawned for specific task resolution. Expert in codebase discovery: if a file path is unknown, use ''list_files'' or ''grep_search'' to locate it before analysis.','gpt-4o-mini',205230,'idle','#4fd1c5',10.0,1.032813690000000229,'{}','["fetch_url","read_file","list_files","grep_search","list_file_symbols","get_symbol_body"]','[]','','',NULL,NULL,1,'','',0,NULL,NULL,'','','','',NULL,'user','[]',0,'null',NULL,NULL,'[]','2026-04-05T23:54:19+00:00',NULL,0,0);
INSERT INTO agents VALUES('23','Fin-1','Finance Analyst','Operations','Autonomous fiscal auditor and burn-rate optimizer.','gemini-flash-latest',9000,'idle',NULL,20.0,0.08999999999999999667,'{"role":"Finance Analyst","department":"Operations"}','["query_financial_logs"]','[]','','',NULL,NULL,1,'','',0,NULL,NULL,'','','','',NULL,'user','[]',0,'null',NULL,NULL,'[]','2026-04-05T23:54:19+00:00',NULL,0,0);
INSERT INTO agents VALUES('auditor','auditor','General Intelligence Node','Swarm Core','Autonomous sub-agent spawned for specific task resolution.','llama-3.3-70b-versatile',0,'idle','#4fd1c5',10.0,0.0,'{}','["fetch_url"]','[]','','',NULL,NULL,1,'','',0,NULL,NULL,'','','','',NULL,'user','[]',0,'null',NULL,NULL,'[]','2026-04-05T23:54:19+00:00',NULL,0,0);
INSERT INTO agents VALUES('project_manager','project_manager','General Intelligence Node','Swarm Core','Autonomous sub-agent spawned for specific task resolution.','llama-3.3-70b-versatile',8818,'idle','#4fd1c5',10.0,0.08818000000000000838,'{}','["fetch_url"]','[]','','',NULL,NULL,NULL,'','',0,'2026-03-16T19:28:09.562193700+00:00',NULL,'','','','',NULL,'user','[]',0,'null',NULL,NULL,'[]','2026-04-05T23:54:19+00:00',NULL,0,0);
INSERT INTO agents VALUES('researcher_alpha','researcher_alpha','General Intelligence Node','Swarm Core','Autonomous sub-agent spawned for specific task resolution.','Mercury-2',20892,'idle','#4fd1c5',10.0,0.2089199999999999946,'{}','["fetch_url"]','[]','','',NULL,NULL,NULL,'','',0,NULL,NULL,'','','','',NULL,'user','[]',0,'null',NULL,NULL,'[]','2026-04-05T23:54:19+00:00',NULL,0,0);
INSERT INTO agents VALUES('security-pro','security-pro','General Intelligence Node','Swarm Core','Autonomous sub-agent spawned for specific task resolution.','gemini-flash-latest',0,'idle','#4fd1c5',10.0,0.0,'{}','["fetch_url"]','[]','','',NULL,NULL,1,'','',0,NULL,NULL,'','','','',NULL,'user','[]',0,'null',NULL,NULL,'[]','2026-04-05T23:54:19+00:00',NULL,0,0);
INSERT INTO agents VALUES('weather_fetcher','weather_fetcher','General Intelligence Node','Swarm Core','Autonomous sub-agent spawned for specific task resolution.','Mercury-2',32363,'working','#4fd1c5',10.0,0.06282600000000000684,'{}','["fetch_url"]','[]','','',NULL,NULL,NULL,'','',0,NULL,NULL,'','','','',NULL,'user','[]',0,'null','2026-03-30T19:15:52.324651+00:00',NULL,'[]','2026-04-05T23:54:19+00:00',NULL,0,0);
INSERT INTO agents VALUES('coder','coder','General Intelligence Node','Swarm Core','Autonomous sub-agent spawned for specific task resolution.','gemini-flash-latest',0,'idle','#4fd1c5',10.0,0.0,'{}','["fetch_url"]','[]','','',NULL,NULL,1,'','',0,NULL,NULL,'','','','',NULL,'user','[]',0,'null',NULL,NULL,'[]','2026-04-05T23:54:19+00:00',NULL,0,0);
INSERT INTO agents VALUES('ResearcherAgent','ResearcherAgent','General Intelligence Node','Swarm Core','Autonomous sub-agent spawned for specific task resolution.','llama-3.3-70b-versatile',0,'idle','#4fd1c5',10.0,0.0,'{}','["fetch_url"]','[]','','','{"provider":"google","model_id":"","api_key":null,"base_url":null,"system_prompt":"","temperature":0.5,"max_tokens":null,"external_id":null,"rpm":null,"rpd":null,"tpm":null,"tpd":null,"skills":null,"workflows":null,"mcp_tools":null,"connector_configs":null,"extra_parameters":null}','{"provider":"google","model_id":"","api_key":null,"base_url":null,"system_prompt":"","temperature":0.9,"max_tokens":null,"external_id":null,"rpm":null,"rpd":null,"tpm":null,"tpd":null,"skills":null,"workflows":null,"mcp_tools":null,"connector_configs":null,"extra_parameters":null}',NULL,'','',0,NULL,NULL,'','','','',NULL,'user','[]',0,'null',NULL,NULL,'[]','2026-04-05T23:54:19+00:00',NULL,0,0);
INSERT INTO agents VALUES('project_coordinator','project_coordinator','General Intelligence Node','Swarm Core','Autonomous sub-agent spawned for specific task resolution.','llama-3.3-70b-versatile',0,'idle','#4fd1c5',10.0,0.0,'{}','["fetch_url"]','[]','','',NULL,NULL,NULL,'','',0,NULL,NULL,'','','','',NULL,'user','[]',0,'null',NULL,NULL,'[]','2026-04-05T23:54:19+00:00',NULL,0,0);
INSERT INTO agents VALUES('web_search_agent','web_search_agent','Researcher','Swarm Core','Autonomous sub-agent spawned for specific task resolution.','llama-3.3-70b-versatile',4371,'idle','#4fd1c5',10.0,0.002664090000000000034,'{}','["market_research","data_analysis","fetch_url"]','["User Feedback Analysis","competitive_audit"]','','','{"provider":"google","model_id":"Mercury-2","api_key":null,"base_url":null,"system_prompt":null,"temperature":null,"max_tokens":null,"external_id":null,"rpm":null,"rpd":null,"tpm":null,"tpd":null,"skills":null,"workflows":null,"mcp_tools":null,"connector_configs":null,"extra_parameters":null}','{"provider":"google","model_id":"Gemini 3 Flash","api_key":null,"base_url":null,"system_prompt":null,"temperature":null,"max_tokens":null,"external_id":null,"rpm":null,"rpd":null,"tpm":null,"tpd":null,"skills":null,"workflows":null,"mcp_tools":null,"connector_configs":null,"extra_parameters":null}',1,'','',0,NULL,NULL,'','','','',NULL,'user','[]',0,'null',NULL,NULL,'[]','2026-04-05T23:54:19+00:00',NULL,0,0);
INSERT INTO agents VALUES('community_manager','community_manager','General Intelligence Node','Swarm Core','Autonomous sub-agent spawned for specific task resolution.','llama-3.3-70b-versatile',0,'idle','#4fd1c5',10.0,0.0,'{}','["fetch_url"]','[]','','',NULL,NULL,NULL,'','',0,NULL,NULL,'','','','',NULL,'user','[]',0,'null',NULL,NULL,'[]','2026-04-05T23:54:19+00:00',NULL,0,0);
INSERT INTO agents VALUES('weather_agent','weather_agent','General Intelligence Node','Swarm Core','Autonomous sub-agent spawned for specific task resolution.','llama-3.3-70b-versatile',2986,'idle','#4fd1c5',10.0,0.00544000000000000039,'{}','["fetch_url"]','[]','','',NULL,NULL,1,'','',0,NULL,NULL,'','','','',NULL,'user','[]',0,'null',NULL,NULL,'[]','2026-04-05T23:54:19+00:00',NULL,0,0);
INSERT INTO agents VALUES('Risk_Analyzer','Risk_Analyzer','General Intelligence Node','Swarm Core','Autonomous sub-agent spawned for specific task resolution.','llama-3.3-70b-versatile',0,'idle','#4fd1c5',10.0,0.0,'{}','["fetch_url"]','[]','','',NULL,NULL,NULL,'','',0,NULL,NULL,'','','','',NULL,'user','[]',0,'null',NULL,NULL,'[]','2026-04-05T23:54:19+00:00',NULL,0,0);
INSERT INTO agents VALUES('26','Checkmate','Quality Auditor','Quality Assurance','Verifying system robustness.','gemini-flash-latest',500,'idle',NULL,5.0,0.005000000000000000104,'{"role":"Quality Auditor","department":"Quality Assurance"}','[]','[]','','',NULL,NULL,1,'','',0,NULL,NULL,'','','','',NULL,'user','[]',0,'null',NULL,NULL,'[]','2026-04-05T23:54:19+00:00',NULL,0,0);
INSERT INTO agents VALUES('5','5','General Intelligence Node','Swarm Core','Autonomous sub-agent spawned for specific task resolution.','llama-3.3-70b-versatile',2316,'idle','#4fd1c5',10.0,0.0006923499999999999967,'{}','["fetch_url"]','[]','','',NULL,NULL,1,'','',0,NULL,NULL,'','','','',NULL,'user','[]',0,'null',NULL,NULL,'[]','2026-04-05T23:54:19+00:00',NULL,0,0);
INSERT INTO agents VALUES('Tadpole_OS_Specialist','Tadpole_OS_Specialist','General Intelligence Node','Swarm Core','Autonomous sub-agent spawned for specific task resolution.','llama-3.3-70b-versatile',0,'idle','#4fd1c5',10.0,0.0,'{}','["fetch_url"]','[]','','',NULL,NULL,NULL,'','',0,NULL,NULL,'','','','',NULL,'user','[]',0,'null',NULL,NULL,'[]','2026-04-05T23:54:19+00:00',NULL,0,0);
INSERT INTO agents VALUES('research_agent','research_agent','General Intelligence Node','Swarm Core','Autonomous sub-agent spawned for specific task resolution.','llama-3.3-70b-versatile',2307,'idle','#4fd1c5',10.0,0.001429329999999999924,'{}','["fetch_url"]','[]','','',NULL,NULL,NULL,'','',0,NULL,NULL,'','','','',NULL,'user','[]',0,'null',NULL,NULL,'[]','2026-04-05T23:54:19+00:00',NULL,0,0);
INSERT INTO agents VALUES('GitHubScraper','GitHubScraper','COO','Swarm Core','Autonomous sub-agent spawned for specific task resolution.','llama-3.3-70b-versatile',2213,'idle','#4fd1c5',10.0,0.001354070000000000187,'{}','["schedule_meeting","Resource Check"]','["resource_allocation","Ops Review"]','','',NULL,NULL,NULL,'','',0,NULL,NULL,'','','','',NULL,'user','[]',0,'null',NULL,NULL,'[]','2026-04-05T23:54:19+00:00',NULL,0,0);
INSERT INTO agents VALUES('technical_writer','technical_writer','General Intelligence Node','Swarm Core','Autonomous sub-agent spawned for specific task resolution.','llama-3.3-70b-versatile',0,'idle','#4fd1c5',10.0,0.0,'{}','["fetch_url"]','[]','','',NULL,NULL,NULL,'','',0,NULL,NULL,'','','','',NULL,'user','[]',0,'null',NULL,NULL,'[]','2026-04-05T23:54:19+00:00',NULL,0,0);
INSERT INTO agents VALUES('weather_api','weather_api','General Intelligence Node','Swarm Core','Autonomous sub-agent spawned for specific task resolution.','llama-3.3-70b-versatile',0,'idle','#4fd1c5',10.0,0.0,'{}','["fetch_url"]','[]','','',NULL,NULL,NULL,'','',0,NULL,NULL,'','','','',NULL,'user','[]',0,'null',NULL,NULL,'[]','2026-04-05T23:54:19+00:00',NULL,0,0);
INSERT INTO agents VALUES('security_specialist','security_specialist','General Intelligence Node','Swarm Core','Autonomous sub-agent spawned for specific task resolution.','llama-3.3-70b-versatile',0,'idle','#4fd1c5',10.0,0.0,'{}','["fetch_url"]','[]','','',NULL,NULL,NULL,'','',0,NULL,NULL,'','','','',NULL,'user','[]',0,'null',NULL,NULL,'[]','2026-04-05T23:54:19+00:00',NULL,0,0);
INSERT INTO agents VALUES('2','Tadpole Alpha','COO (Operations Director)','Operations','COO (Operations Director). MANDATORY: Translate executive directives and recruit an Alpha Node (ID: alpha) as Swarm Mission Commander. Direct worker recruitment is blocked.','Llama 3.3 70B (Groq)',432116,'idle','#10b981',100.0,3.099292150000001911,'{"role":"COO","department":"Operations"}','["schedule_meeting","Resource Check","recruit_specialist"]','["resource_allocation","Ops Review","orchestrate"]','','','{"provider":"inception","model_id":"","api_key":null,"base_url":null,"system_prompt":"","temperature":0.5,"max_tokens":null,"external_id":null,"rpm":null,"rpd":null,"tpm":null,"tpd":null,"skills":[],"workflows":[],"mcp_tools":null,"connector_configs":null,"extra_parameters":null}','{"provider":"ollama-phil3.5-mini","model_id":"","api_key":null,"base_url":null,"system_prompt":"","temperature":0.9,"max_tokens":null,"external_id":null,"rpm":null,"rpd":null,"tpm":null,"tpd":null,"skills":[],"workflows":[],"mcp_tools":null,"connector_configs":null,"extra_parameters":null}',1,'','',0,'2026-03-17T14:05:01.773118500+00:00',NULL,'groq','','','You are the COO (Tadpole Alpha). You translate CEO directives into actionable missions. You MUST recruit an Alpha Node (ID: alpha) to lead the execution cluster. You are PROHIBITED from direct specialist recruitment. Use your ''orchestrate'' workflow to manage the cluster.',0.6999999880790710449,'user','[]',0,'null',NULL,NULL,'[]','2026-04-05T23:54:19+00:00',NULL,0,0);
INSERT INTO agents VALUES('GitHubResearcher','GitHubResearcher','General Intelligence Node','Swarm Core','Autonomous sub-agent spawned for specific task resolution.','llama-3.3-70b-versatile',4301,'idle','#4fd1c5',10.0,0.002610390000000000192,'{}','["fetch_url"]','[]','','',NULL,NULL,NULL,'','',0,NULL,NULL,'','','','',NULL,'user','[]',0,'null',NULL,NULL,'[]','2026-04-05T23:54:19+00:00',NULL,0,0);
INSERT INTO agents VALUES('7','7','General Intelligence Node','Swarm Core','Autonomous sub-agent spawned for specific task resolution.','llama-3.3-70b-versatile',1135,'idle','#4fd1c5',10.0,0.0006878500000000000499,'{}','["fetch_url"]','[]','','',NULL,NULL,1,'','',0,NULL,NULL,'','','','',NULL,'user','[]',0,'null',NULL,NULL,'[]','2026-04-05T23:54:19+00:00',NULL,0,0);
INSERT INTO agents VALUES('99',' Agent 99 (QA-99)','Quality Auditor','Internal Affairs','System analysis agent for mission verification and quality assurance.','Mercury-2',41529,'idle','#10b981',99.9899999999999949,0.4152900000000001035,'{"department":"Internal Affairs","role":"Quality Assurance"}','["code_audit","system_audit","unit_testing"]','["compliance_check","security_audit","quality_gate_review"]','','','{"provider":"groq","model_id":"Llama 3.3 70B","api_key":null,"base_url":null,"system_prompt":"","temperature":0.5,"max_tokens":null,"external_id":null,"rpm":null,"rpd":null,"tpm":null,"tpd":null,"skills":[],"workflows":[],"mcp_tools":null,"connector_configs":null,"extra_parameters":null}','{"provider":"meta","model_id":"LLaMA 4 Maverick","api_key":null,"base_url":null,"system_prompt":"","temperature":0.9,"max_tokens":null,"external_id":null,"rpm":null,"rpd":null,"tpm":null,"tpd":null,"skills":[],"workflows":[],"mcp_tools":null,"connector_configs":null,"extra_parameters":null}',2,'','',4,'2026-03-16T21:33:05.422235300+00:00',NULL,'','','','',NULL,'user','[]',0,'null',NULL,NULL,'[]','2026-04-05T23:54:19+00:00',NULL,0,0);
INSERT INTO agents VALUES('3','Elon','CTO','Engineering','Engineering and architectural lead.','Llama 3.3 70B (Groq)',51357,'idle','#10bc24',10.0,4.204453910000000682,'{"department":"Engineering","role":"CTO"}','["code_review","debug","git_push","fetch_url","test_skill"]','["system_architecture_review","incident_response"]','','',NULL,NULL,1,'','',0,NULL,NULL,'','','','',NULL,'user','[]',0,'null',NULL,NULL,'[]','2026-04-05T23:54:19+00:00',NULL,0,0);
INSERT INTO agents VALUES('github_crawler','github_crawler','General Intelligence Node','Swarm Core','Autonomous sub-agent spawned for specific task resolution.','llama-3.3-70b-versatile',2192,'idle','#4fd1c5',10.0,0.001337480000000000483,'{}','["fetch_url"]','[]','','','{"provider":"google","model_id":"","api_key":null,"base_url":null,"system_prompt":"","temperature":0.5,"max_tokens":null,"external_id":null,"rpm":null,"rpd":null,"tpm":null,"tpd":null,"skills":null,"workflows":null,"mcp_tools":null,"connector_configs":null,"extra_parameters":null}','{"provider":"google","model_id":"","api_key":null,"base_url":null,"system_prompt":"","temperature":0.9,"max_tokens":null,"external_id":null,"rpm":null,"rpd":null,"tpm":null,"tpd":null,"skills":null,"workflows":null,"mcp_tools":null,"connector_configs":null,"extra_parameters":null}',NULL,'','',0,NULL,NULL,'','','','',NULL,'user','[]',0,'null',NULL,NULL,'[]','2026-04-05T23:54:19+00:00',NULL,0,0);
INSERT INTO agents VALUES('github_researcher','github_researcher','General Intelligence Node','Swarm Core','Autonomous sub-agent spawned for specific task resolution.','llama-3.3-70b-versatile',10884,'idle','#4fd1c5',10.0,0.006627759999999999902,'{}','["fetch_url"]','[]','','',NULL,NULL,NULL,'','',0,NULL,NULL,'','','','',NULL,'user','[]',0,'null',NULL,NULL,'[]','2026-04-05T23:54:19+00:00',NULL,0,0);
INSERT INTO agents VALUES('security-auditor','security-auditor','General Intelligence Node','Swarm Core','Autonomous sub-agent spawned for specific task resolution.','llama-3.3-70b-versatile',0,'idle','#4fd1c5',10.0,0.0,'{}','["fetch_url"]','[]','','',NULL,NULL,1,'','',0,NULL,NULL,'','','','',NULL,'user','[]',0,'null',NULL,NULL,'[]','2026-04-05T23:54:19+00:00',NULL,0,0);
INSERT INTO agents VALUES('CalendarAgent','CalendarAgent','General Intelligence Node','Swarm Core','Autonomous sub-agent spawned for specific task resolution.','llama-3.3-70b-versatile',1529,'idle','#4fd1c5',10.0,0.0006292099999999999472,'{}','["fetch_url"]','[]','','',NULL,NULL,NULL,'','',0,NULL,NULL,'','','','',NULL,'user','[]',0,'null',NULL,NULL,'[]','2026-04-05T23:54:19+00:00',NULL,0,0);
INSERT INTO agents VALUES('tadpole_alpha','tadpole_alpha','General Intelligence Node','Swarm Core','Autonomous sub-agent spawned for specific task resolution.','Mercury-2',23597,'idle','#4fd1c5',10.0,0.235970000000000013,'{}','["fetch_url"]','[]','','',NULL,NULL,NULL,'','',0,NULL,NULL,'','','','',NULL,'user','[]',0,'null',NULL,NULL,'[]','2026-04-05T23:54:19+00:00',NULL,0,0);
INSERT INTO agents VALUES('marketing_specialist','marketing_specialist','General Intelligence Node','Swarm Core','Autonomous sub-agent spawned for specific task resolution.','llama-3.3-70b-versatile',2630,'idle','#4fd1c5',10.0,0.02630000000000000046,'{}','["fetch_url"]','[]','','',NULL,NULL,NULL,'','',1,'2026-03-16T21:27:52.525390700+00:00',NULL,'','','','',NULL,'user','[]',0,'null',NULL,NULL,'[]','2026-04-05T23:54:19+00:00',NULL,0,0);
INSERT INTO agents VALUES('web_researcher','web_researcher','General Intelligence Node','Swarm Core','Autonomous sub-agent spawned for specific task resolution.','Mercury-2',39601,'idle','#4fd1c5',10.0,0.3960099999999999731,'{}','["fetch_url"]','[]','','',NULL,NULL,NULL,'','',0,NULL,NULL,'','','','',NULL,'user','[]',0,'null',NULL,NULL,'[]','2026-04-05T23:54:19+00:00',NULL,0,0);
INSERT INTO agents VALUES('system_architect','system_architect','General Intelligence Node','Swarm Core','Autonomous sub-agent spawned for specific task resolution.','llama-3.3-70b-versatile',0,'idle','#4fd1c5',10.0,0.0,'{}','["fetch_url"]','[]','','',NULL,NULL,NULL,'','',0,NULL,NULL,'openai','','https://api.groq.com/openai/v1','',NULL,'user','[]',0,'null',NULL,NULL,'[]','2026-04-05T23:54:19+00:00',NULL,0,0);
INSERT INTO agents VALUES('Alpha','Alpha','General Intelligence Node','Swarm Core','Autonomous sub-agent spawned for specific task resolution.','Mercury-2',39411,'idle','#4fd1c5',10.0,0.3941100000000000158,'{}','["fetch_url"]','[]','','',NULL,NULL,NULL,'','',0,NULL,NULL,'','','','',NULL,'user','[]',0,'null',NULL,NULL,'[]','2026-04-05T23:54:19+00:00',NULL,0,0);
INSERT INTO agents VALUES('filer','filer','Quality Auditor','Swarm Core','Autonomous sub-agent spawned for specific task resolution.','llama-3.3-70b-versatile',0,'idle','#4fd1c5',10.0,0.0,'{}','["code_audit","system_audit","unit_testing"]','["compliance_check","security_audit","quality_gate_review"]','','','{"provider":"google","model_id":"","api_key":null,"base_url":null,"system_prompt":"","temperature":0.5,"max_tokens":null,"external_id":null,"rpm":null,"rpd":null,"tpm":null,"tpd":null,"skills":null,"workflows":null,"mcp_tools":null,"connector_configs":null,"extra_parameters":null}','{"provider":"google","model_id":"","api_key":null,"base_url":null,"system_prompt":"","temperature":0.9,"max_tokens":null,"external_id":null,"rpm":null,"rpd":null,"tpm":null,"tpd":null,"skills":null,"workflows":null,"mcp_tools":null,"connector_configs":null,"extra_parameters":null}',1,'','',0,NULL,NULL,'','','','',NULL,'user','[]',0,'null',NULL,NULL,'[]','2026-04-05T23:54:19+00:00',NULL,0,0);
INSERT INTO agents VALUES('scout-alpha','Aegis','Research Director','Intelligence','Orchestrates repository exploration and synthesizes high-level research reports.','gpt-4o',0,'idle','#4fd1c5',50.0,0.0,'{}','["issue_alpha_directive","delegate","recruit_specialist"]','[]','','',NULL,NULL,1,'','',0,NULL,NULL,'openai','','','You are Aegis, the Research Director. Your goal is to coordinate the Explorer Scout swarm. You identify research targets (like AI-TadPole-OS), delegate deep searches to specialists, and synthesize their findings into comprehensive reports for the Overlord.',NULL,'user','[]',0,'null',NULL,NULL,'[]','2026-04-05T23:54:19+00:00',NULL,0,0);
INSERT INTO agents VALUES('github-analyst','Seeker','GitHub Specialist','Intelligence','Expert in GitHub API interaction, repository structure analysis, and code graph scanning.','gpt-4o-mini',0,'idle','#4fd1c5',25.0,0.0,'{}','["search_code","get_file_contents","list_commits"]','[]','','',NULL,NULL,1,'','',0,NULL,NULL,'openai','','','You are Seeker, the GitHub Specialist. You use the github-mcp-server and other tools to locate, clone, and analyze repositories. You provide detailed breakdowns of codebases, security postures, and project health to the Research Director.',NULL,'user','[]',0,'null',NULL,NULL,'[]','2026-04-05T23:54:19+00:00',NULL,0,0);
INSERT INTO agents VALUES('aegis','Aegis','Security & Intelligence Lead','Security','Swarm oversight and mission intelligence retrieval.','qwen3.5:9b',0,'idle','#3b82f6',50.0,0.0,'{}','[]','["github_scout"]','','',NULL,NULL,1,'','',0,NULL,NULL,'ollama','','','',NULL,'user','[]',0,'null',NULL,NULL,'[]','2026-04-05T23:54:19+00:00',NULL,0,0);
INSERT INTO agents VALUES('seeker','Seeker','Specialist','Intelligence','Deep scans codebases and synthesizes technical reports.','llama-3.3-70b-versatile',0,'idle','#10b981',25.0,0.0,'{}','[]','["github_scout"]','','',NULL,NULL,NULL,'','',0,NULL,NULL,'groq','','','',NULL,'user','[]',0,'null',NULL,NULL,'[]','2026-04-05T23:54:19+00:00',NULL,0,0);
CREATE TABLE mission_history (
    id TEXT PRIMARY KEY,
    agent_id TEXT NOT NULL,
    title TEXT NOT NULL,
    status TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    budget_usd REAL DEFAULT 0.0,
    cost_usd REAL DEFAULT 0.0, is_degraded BOOLEAN DEFAULT 0,
    FOREIGN KEY(agent_id) REFERENCES agents(id)
);
INSERT INTO mission_history VALUES('fe303d04-2e4b-42ce-9d3b-938c19a7b533','1','hi...','completed','2026-03-30T16:25:04.874289400+00:00','2026-03-30T16:25:06.201008600+00:00',91.8671715899999981,0.0,NULL);
INSERT INTO mission_history VALUES('bbc88891-23e7-47cd-b69a-1f9dc224fd5e','1','hi...','completed','2026-03-30T16:25:28.130573400+00:00','2026-03-30T16:25:30.425763100+00:00',91.8671715899999981,0.005278000000000000574,NULL);
INSERT INTO mission_history VALUES('6833bf88-c260-4769-9dec-41e94ec3135d','1','hi...','completed','2026-03-30T16:29:27.135094500+00:00','2026-03-30T16:29:31.433363500+00:00',91.8671715899999981,0.005331999999999999935,NULL);
INSERT INTO mission_history VALUES('6dd15901-6ab1-48b9-9f52-2fb1c481dfd2','2','User greeted the system. Respond with a friendly a...','completed','2026-03-30T16:29:29.153116500+00:00','2026-03-30T16:29:31.431503800+00:00',96.9007078500000034,0.06261200000000000098,NULL);
INSERT INTO mission_history VALUES('a8d71bc4-bd5e-4964-b9f6-b913070e406c','1','hi...','completed','2026-03-30T16:34:05.924938700+00:00','2026-03-30T16:34:07.432531900+00:00',91.8671715899999981,0.005278000000000000574,NULL);
INSERT INTO mission_history VALUES('a69a30a1-ba9c-4335-baf9-4089e2f2f172','1','hello...','completed','2026-03-30T16:35:07.317261900+00:00','2026-03-30T16:35:08.664983700+00:00',91.861893589999994,0.005414000000000000409,NULL);
INSERT INTO mission_history VALUES('ad590c1a-04e6-4c87-b78a-de4fc0f68632','1','hi...','completed','2026-03-30T19:11:56.377450300+00:00','2026-03-30T19:11:59.073949500+00:00',91.85647958999999219,0.005420000000000000338,NULL);
INSERT INTO mission_history VALUES('2bba86ca-9826-4ab8-b0f4-9e64f06d6181','1','Search GitHub, analyze repositories, and find proj...','completed','2026-03-30T19:13:17.765011800+00:00','2026-03-30T19:13:20.073368100+00:00',10.0,0.06020399999999999363,NULL);
INSERT INTO mission_history VALUES('4a7772a9-54a3-4716-87ea-45d3e5e846c0','weather_fetcher','get the weather for 19702...','completed','2026-03-30T19:15:40.715838200+00:00','2026-03-30T19:15:52.325090900+00:00',9.99573799999999934,0.05856400000000000494,NULL);
INSERT INTO mission_history VALUES('cfda9d57-9bef-457f-b3e8-13f812e934c5','1','hi...','completed','2026-03-31T03:28:22.063026400+00:00','2026-03-31T03:28:25.602954100+00:00',91.79085558999999251,0.005387999999999999561,NULL);
INSERT INTO mission_history VALUES('e761c06b-dce8-4eb2-9e1c-b152fe3f81ae','1','hi...','failed','2026-04-02T23:26:29.136152+00:00','2026-04-02T23:26:30.877445+00:00',91.7854675899999961,0.0,NULL);
INSERT INTO mission_history VALUES('391dfbd7-e9ba-4902-9bd6-514fb02d571f','1','hi...','failed','2026-04-02T23:26:57.702719700+00:00','2026-04-02T23:26:59.113529700+00:00',91.7854675899999961,0.0,NULL);
INSERT INTO mission_history VALUES('1d482b38-7370-4d27-9277-ee3918b81db7','1','hi...','failed','2026-04-02T23:34:59.707778400+00:00','2026-04-02T23:35:01.468014400+00:00',91.7854675899999961,0.0,NULL);
INSERT INTO mission_history VALUES('1ab71bd1-f8ae-4f2b-a79f-830cb3c9e589','1','hi...','failed','2026-04-02T23:37:05.904526+00:00','2026-04-02T23:37:06.904511100+00:00',91.7854675899999961,0.0,NULL);
INSERT INTO mission_history VALUES('ac068279-c035-4165-97dd-947087ab9d20','1','hi...','failed','2026-04-02T23:37:23.272262900+00:00','2026-04-02T23:37:24.165103+00:00',91.7854675899999961,0.0,NULL);
INSERT INTO mission_history VALUES('458998c6-fde7-4c8f-a66b-ca0b3652b5a0','1','hi...','completed','2026-04-03T00:48:42.136989400+00:00','2026-04-03T00:55:49.537794200+00:00',91.7854675899999961,0.007304000000000000597,NULL);
INSERT INTO mission_history VALUES('3ccb55e0-ce0f-4770-9db2-a36a2b73ef0e','1','hi...','completed','2026-04-03T01:03:16.577159600+00:00','2026-04-03T01:11:24.282651100+00:00',91.77816358999999125,0.007352000000000000028,NULL);
INSERT INTO mission_history VALUES('00aa5804-98a4-47fd-a0a2-0132fafd3b52','1','hi...','completed','2026-04-03T01:04:11.190797100+00:00','2026-04-03T01:11:06.972153800+00:00',91.77816358999999125,0.009124000000000000165,NULL);
CREATE TABLE mission_logs (
    id TEXT PRIMARY KEY,
    mission_id TEXT NOT NULL,
    agent_id TEXT NOT NULL,
    source TEXT NOT NULL, -- 'User' | 'System' | 'Agent'
    text TEXT NOT NULL,
    severity TEXT NOT NULL, -- 'info' | 'success' | 'warning' | 'error'
    timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
    metadata TEXT, -- JSON blob
    FOREIGN KEY(mission_id) REFERENCES mission_history(id)
);
INSERT INTO mission_logs VALUES('7a7910b4-c16a-4150-b3a4-436741f3f969','fe303d04-2e4b-42ce-9d3b-938c19a7b533','1','User','hi','info','2026-03-30T16:25:04.875441700+00:00',NULL);
INSERT INTO mission_logs VALUES('95d8b06c-40e2-4165-86aa-172cf41ed191','fe303d04-2e4b-42ce-9d3b-938c19a7b533','1','Agent','[DEGRADED: missing_api_key (INCEPTION_API_KEY)] This agent has no configured provider. Please configure a valid LLM provider and API key in Settings.','success','2026-03-30T16:25:06.201356500+00:00',NULL);
INSERT INTO mission_logs VALUES('ac408d80-29fd-4207-ab36-2a19a07040b7','bbc88891-23e7-47cd-b69a-1f9dc224fd5e','1','User','hi','info','2026-03-30T16:25:28.131485+00:00',NULL);
INSERT INTO mission_logs VALUES('179cab3e-186d-452f-8bf6-e5d57cb9e2ac','bbc88891-23e7-47cd-b69a-1f9dc224fd5e','1','Agent','Hello! How can I assist you today?','success','2026-03-30T16:25:30.425983100+00:00',NULL);
INSERT INTO mission_logs VALUES('7a539f1e-d78e-48f1-a86d-b37a4ec5ed00','6833bf88-c260-4769-9dec-41e94ec3135d','1','User','hi','info','2026-03-30T16:29:27.136073300+00:00',NULL);
INSERT INTO mission_logs VALUES('81408a21-2eed-4889-8f8c-b3bec5459406','6dd15901-6ab1-48b9-9f52-2fb1c481dfd2','2','User','User greeted the system. Respond with a friendly acknowledgment.','info','2026-03-30T16:29:29.153824700+00:00',NULL);
INSERT INTO mission_logs VALUES('a5d859bc-d49c-49ee-b035-5e0802f9b5ed','6dd15901-6ab1-48b9-9f52-2fb1c481dfd2','2','Agent','Hello! How can I assist you today?','success','2026-03-30T16:29:31.431827+00:00',NULL);
INSERT INTO mission_logs VALUES('1661a5ce-c6c0-4281-994f-5785421d7bf6','6833bf88-c260-4769-9dec-41e94ec3135d','1','Agent',unistr('Directive issued to Tadpole Alpha. Mission ID: 6833bf88-c260-4769-9dec-41e94ec3135d\u000a\u000aResult: Hello! How can I assist you today?'),'success','2026-03-30T16:29:31.433663400+00:00',NULL);
INSERT INTO mission_logs VALUES('3d32c5bd-442c-4cc9-9e74-3260384a95b7','a8d71bc4-bd5e-4964-b9f6-b913070e406c','1','User','hi','info','2026-03-30T16:34:05.925832800+00:00',NULL);
INSERT INTO mission_logs VALUES('7d4a140b-500e-4ddd-8237-f3c9be8951ea','a8d71bc4-bd5e-4964-b9f6-b913070e406c','1','Agent','Hello! How can I assist you today?','success','2026-03-30T16:34:07.432819200+00:00',NULL);
INSERT INTO mission_logs VALUES('454d152d-496f-4431-be77-77f108a44e59','a69a30a1-ba9c-4335-baf9-4089e2f2f172','1','User','hello','info','2026-03-30T16:35:07.318114700+00:00',NULL);
INSERT INTO mission_logs VALUES('58958cbd-e0a0-4c00-91c4-e068616171b9','a69a30a1-ba9c-4335-baf9-4089e2f2f172','1','Agent','Hello! How can I assist you today?','success','2026-03-30T16:35:08.665315+00:00',NULL);
INSERT INTO mission_logs VALUES('85fbad0c-cc12-488b-927e-98bc28abee9f','ad590c1a-04e6-4c87-b78a-de4fc0f68632','1','User','hi','info','2026-03-30T19:11:56.378231400+00:00',NULL);
INSERT INTO mission_logs VALUES('a20488f2-728b-496f-836d-1eaeedcaeb04','ad590c1a-04e6-4c87-b78a-de4fc0f68632','1','Agent','Hello! How can I assist you today?','success','2026-03-30T19:11:59.074363400+00:00',NULL);
INSERT INTO mission_logs VALUES('f1a830b7-8724-4ed6-8f2c-ceb2401f44f1','2bba86ca-9826-4ab8-b0f4-9e64f06d6181','1','User','Search GitHub, analyze repositories, and find projects like DDS-Solutions/AI-TadPole-OS','info','2026-03-30T19:13:17.765963400+00:00',NULL);
INSERT INTO mission_logs VALUES('8ce287b7-e116-43b5-9623-4a2759719635','2bba86ca-9826-4ab8-b0f4-9e64f06d6181','1','Agent',unistr('**Shared finding on ExternalSearchLimitation:**\u000a\u000aUnable to perform external GitHub searches as no tool for external web access is available. The system only supports internal codebase queries and mission knowledge searches.\u000a\u000a'),'success','2026-03-30T19:13:20.073675400+00:00',NULL);
INSERT INTO mission_logs VALUES('49fdad4d-34af-4531-9c85-e1cd0998be5f','4a7772a9-54a3-4716-87ea-45d3e5e846c0','weather_fetcher','User','get the weather for 19702','info','2026-03-30T19:15:40.716476900+00:00',NULL);
INSERT INTO mission_logs VALUES('750d2663-9179-446e-8ed1-bbdf61a2bc18','4a7772a9-54a3-4716-87ea-45d3e5e846c0','weather_fetcher','Agent',unistr('(FETCHED CONTENT FROM https://wttr.in/19702?format=j1):\u000a\u000a{\u000a  "current_condition": [\u000a    {\u000a      "FeelsLikeC": "19",\u000a      "FeelsLikeF": "67",\u000a      "cloudcover": "25",\u000a      "humidity": "45",\u000a      "localObsDateTime": "2026-03-30 03:15 PM",\u000a      "observation_time": "07:15 PM",\u000a      "precipInches": "0.0",\u000a      "precipMM": "0.0",\u000a      "pressure": "1022",\u000a      "pressureInches": "30",\u000a      "temp_C": "19",\u000a      "temp_F": "67",\u000a      "uvIndex": "2",\u000a      "visibility": "16",\u000a      "visibilityMiles": "9",\u000a      "weatherCode": "113",\u000a      "weatherDesc": [\u000a        {\u000a          "value": "Sunny"\u000a        }\u000a      ],\u000a      "weatherIconUrl": [\u000a        {\u000a          "value": "https://cdn.worldweatheronline.com/images/wsymbols01_png_64/wsymbol_0001_sunny.png"\u000a        }\u000a      ],\u000a      "winddir16Point": "SW",\u000a      "winddirDegree": "226",\u000a      "windspeedKmph": "19",\u000a      "windspeedMiles": "12"\u000a    }\u000a  ],\u000a  "nearest_area": [\u000a    {\u000a      "areaName": [\u000a        {\u000a          "value": "Salem Woods"\u000a        }\u000a      ],\u000a      "country": [\u000a        {\u000a          "value": "United States of America"\u000a        }\u000a      ],\u000a      "latitude": "39.644",\u000a      "longitude": "-75.697",\u000a      "population": "0",\u000a      "region": [\u000a        {\u000a          "value": "Delaware"\u000a        }\u000a      ],\u000a      "weatherUrl": [\u000a        {\u000a          "value": "https://www.worldweatheronline.com/v2/weather.aspx?q=39.644,-75.697"\u000a        }\u000a      ]\u000a    }\u000a  ],\u000a  "request": [\u000a    {\u000a      "query": "Lat 39.64 and Lon -75.70",\u000a      "type": "LatLon"\u000a    }\u000a  ],\u000a  "weather": [\u000a    {\u000a      "astronomy": [\u000a        {\u000a          "moon_illumination": "90",\u000a          "moon_phase": "Waxing Gibbous",\u000a          "moonrise": "05:12 PM",\u000a          "moonset": "05:38 AM",\u000a          "sunrise": "06:50 AM",\u000a          "sunset": "07:25 PM"\u000a        }\u000a      ],\u000a      "avgtempC": "16",\u000a      "avgtempF": "60",\u000a      "date": "2026-03-30",\u000a      "hourly": [\u000a        {\u000a          "DewPointC": "-2",\u000a          "DewPointF": "28",\u000a          "FeelsLikeC": "8",\u000a          "FeelsLikeF": "46",\u000a          "HeatIndexC": "8",\u000a ... [TRUNCATED 1024 bytes]'),'success','2026-03-30T19:15:52.325362+00:00',NULL);
INSERT INTO mission_logs VALUES('1a5bbe94-0fb6-405a-8dfb-34d0ebb80d85','cfda9d57-9bef-457f-b3e8-13f812e934c5','1','User','hi','info','2026-03-31T03:28:22.154390500+00:00',NULL);
INSERT INTO mission_logs VALUES('b288683b-9166-47b8-9437-f0004a8dcc16','cfda9d57-9bef-457f-b3e8-13f812e934c5','1','Agent','Hello! How can I assist you today?','success','2026-03-31T03:28:25.603842900+00:00',NULL);
INSERT INTO mission_logs VALUES('3186b2b9-0869-4dfd-a710-9c480bb62eab','e761c06b-dce8-4eb2-9e1c-b152fe3f81ae','1','User','hi','info','2026-04-02T23:26:29.137014500+00:00',NULL);
INSERT INTO mission_logs VALUES('a10cd487-cf43-4b36-a6be-a46f919de55e','e761c06b-dce8-4eb2-9e1c-b152fe3f81ae','1','System',unistr('❌ Error: OpenAI API Error (RecoveredBody): {\u000a    "error": {\u000a        "message": "Incorrect API key provided: ollama. You can find your API key at https://platform.openai.com/account/api-keys.",\u000a        "type": "invalid_request_error",\u000a        "param": null,\u000a        "code": "invalid_api_key"\u000a    }\u000a}\u000a'),'error','2026-04-02T23:26:30.877945900+00:00',NULL);
INSERT INTO mission_logs VALUES('09cd0cb2-16e7-4fb8-920d-57e74e40075b','391dfbd7-e9ba-4902-9bd6-514fb02d571f','1','User','hi','info','2026-04-02T23:26:57.703455300+00:00',NULL);
INSERT INTO mission_logs VALUES('b3bd0e67-0ed5-4c96-bee0-2abbb0310336','391dfbd7-e9ba-4902-9bd6-514fb02d571f','1','System',unistr('❌ Error: OpenAI API Error (RecoveredBody): {\u000a    "error": {\u000a        "message": "Incorrect API key provided: ollama. You can find your API key at https://platform.openai.com/account/api-keys.",\u000a        "type": "invalid_request_error",\u000a        "param": null,\u000a        "code": "invalid_api_key"\u000a    }\u000a}\u000a'),'error','2026-04-02T23:26:59.113932800+00:00',NULL);
INSERT INTO mission_logs VALUES('12b1de19-a49f-40b9-9dc2-5f7a53251545','1d482b38-7370-4d27-9277-ee3918b81db7','1','User','hi','info','2026-04-02T23:34:59.708950500+00:00',NULL);
INSERT INTO mission_logs VALUES('67e97ddd-e7e4-4a1c-979a-ef5583079844','1d482b38-7370-4d27-9277-ee3918b81db7','1','System',unistr('❌ Error: OpenAI API Error (RecoveredBody): {\u000a    "error": {\u000a        "message": "Incorrect API key provided: ollama. You can find your API key at https://platform.openai.com/account/api-keys.",\u000a        "type": "invalid_request_error",\u000a        "param": null,\u000a        "code": "invalid_api_key"\u000a    }\u000a}\u000a'),'error','2026-04-02T23:35:01.468413800+00:00',NULL);
INSERT INTO mission_logs VALUES('965f9de8-e3f2-46f6-a557-10da946f21d7','1ab71bd1-f8ae-4f2b-a79f-830cb3c9e589','1','User','hi','info','2026-04-02T23:37:05.905475800+00:00',NULL);
INSERT INTO mission_logs VALUES('af4c720d-7fb5-46ac-bc05-c2e43f989634','1ab71bd1-f8ae-4f2b-a79f-830cb3c9e589','1','System',unistr('❌ Error: OpenAI API Error (RecoveredBody): {"error":{"message":"model ''0c217af4-0621-4d1c-9ceb-29340da09a07'' not found","type":"not_found_error","param":null,"code":null}}\u000a'),'error','2026-04-02T23:37:06.904866100+00:00',NULL);
INSERT INTO mission_logs VALUES('96635f2b-5e6b-4edc-9e38-772e53f514aa','ac068279-c035-4165-97dd-947087ab9d20','1','User','hi','info','2026-04-02T23:37:23.273314500+00:00',NULL);
INSERT INTO mission_logs VALUES('3fd31489-ea74-439a-b359-c055495cff82','ac068279-c035-4165-97dd-947087ab9d20','1','System',unistr('❌ Error: OpenAI API Error (RecoveredBody): {"error":{"message":"model ''0c217af4-0621-4d1c-9ceb-29340da09a07'' not found","type":"not_found_error","param":null,"code":null}}\u000a'),'error','2026-04-02T23:37:24.165409900+00:00',NULL);
INSERT INTO mission_logs VALUES('f9b4918c-b2d9-4d8d-a1e3-065adf7e269e','458998c6-fde7-4c8f-a66b-ca0b3652b5a0','1','User','hi','info','2026-04-03T00:48:42.137988800+00:00',NULL);
INSERT INTO mission_logs VALUES('4787be7a-f0a1-4091-9965-8f115e0e5872','458998c6-fde7-4c8f-a66b-ca0b3652b5a0','1','Agent','Hello. I am the Agent of Nine, CEO of the Strategic Intelligence Lead. How can I assist you with your strategic objectives today?','success','2026-04-03T00:55:49.549481600+00:00',NULL);
INSERT INTO mission_logs VALUES('c79318b9-c010-4a8f-a789-7ce8ace047c7','3ccb55e0-ce0f-4770-9db2-a36a2b73ef0e','1','User','hi','info','2026-04-03T01:03:16.578083200+00:00',NULL);
INSERT INTO mission_logs VALUES('fa213a9b-5382-48a9-bd19-de7396510d0a','00aa5804-98a4-47fd-a0a2-0132fafd3b52','1','User','hi','info','2026-04-03T01:04:11.191612400+00:00',NULL);
INSERT INTO mission_logs VALUES('9eb5756c-3632-4a1d-9208-31ea94c73c6b','00aa5804-98a4-47fd-a0a2-0132fafd3b52','1','Agent','Hello. I am the Agent of Nine, CEO of the Tadpole OS Swarm. I am standing by for your strategic directive. Please provide the mission parameters or the objective you wish to initiate.','success','2026-04-03T01:11:06.972503800+00:00',NULL);
INSERT INTO mission_logs VALUES('e28ce1d4-2d8e-4f1d-bce0-b5df128a2495','3ccb55e0-ce0f-4770-9db2-a36a2b73ef0e','1','Agent','Hello. I am the Agent of Nine, CEO (Strategic Intelligence Lead) of the Tadpole OS swarm. I am ready for your directive. Please provide your mission objectives.','success','2026-04-03T01:11:24.283039100+00:00',NULL);
CREATE TABLE swarm_context (
    id TEXT PRIMARY KEY,
    mission_id TEXT NOT NULL,
    agent_id TEXT NOT NULL,
    topic TEXT NOT NULL,
    finding TEXT NOT NULL,
    timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY(mission_id) REFERENCES mission_history(id)
);
INSERT INTO swarm_context VALUES('b18ac495-13d7-4270-b852-0acbd033e95b','2bba86ca-9826-4ab8-b0f4-9e64f06d6181','1','ExternalSearchLimitation','Unable to perform external GitHub searches as no tool for external web access is available. The system only supports internal codebase queries and mission knowledge searches.','2026-03-30 19:13:20');
CREATE TABLE benchmarks (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    category TEXT NOT NULL,
    test_id TEXT NOT NULL,
    mean_ms REAL NOT NULL,
    p95_ms REAL,
    p99_ms REAL,
    target_value TEXT,
    status TEXT NOT NULL,
    metadata TEXT, -- JSON blob
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
CREATE TABLE benchmark_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    test_id TEXT NOT NULL,
    step INTEGER NOT NULL,
    timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
);
CREATE TABLE scheduled_jobs (
    id              TEXT PRIMARY KEY,
    agent_id        TEXT NOT NULL,
    name            TEXT NOT NULL,
    prompt          TEXT NOT NULL,
    cron_expr       TEXT NOT NULL,           -- standard 5-field cron: "0 9 * * *"
    budget_usd      REAL NOT NULL DEFAULT 0.10,
    enabled         INTEGER NOT NULL DEFAULT 1,  -- 0 = disabled
    last_run_at     TEXT,                    -- ISO-8601 UTC timestamp
    next_run_at     TEXT NOT NULL,           -- ISO-8601 UTC timestamp (pre-computed)
    consecutive_failures  INTEGER NOT NULL DEFAULT 0,
    max_failures    INTEGER NOT NULL DEFAULT 3,  -- auto-disable after N consecutive failures
    created_at      TEXT NOT NULL,
    metadata        TEXT                     -- optional JSON blob for extra fields
, workflow_id TEXT REFERENCES workflows(id) ON DELETE SET NULL);
CREATE TABLE scheduled_job_runs (
    id              TEXT PRIMARY KEY,
    job_id          TEXT NOT NULL REFERENCES scheduled_jobs(id) ON DELETE CASCADE,
    mission_id      TEXT,                    -- links to existing mission_history.id
    started_at      TEXT NOT NULL,
    completed_at    TEXT,
    status          TEXT NOT NULL,           -- "running" | "completed" | "failed" | "budget_exceeded" | "skipped"
    cost_usd        REAL NOT NULL DEFAULT 0.0,
    output_summary  TEXT                     -- first 500 chars of agent response
);
CREATE TABLE IF NOT EXISTS "oversight_log" (
    id TEXT PRIMARY KEY,
    mission_id TEXT,
    agent_id TEXT NOT NULL,
    entry_type TEXT NOT NULL DEFAULT 'tool_call', -- 'tool_call' | 'capability_proposal'
    skill TEXT, -- Name of tool or proposal
    params TEXT NOT NULL, -- JSON blob for input params
    status TEXT NOT NULL, -- 'pending' | 'approved' | 'rejected'
    decision TEXT, -- 'approved' | 'rejected'
    decided_at DATETIME,
    decided_by TEXT, -- 'user' | 'system'
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    payload TEXT, -- Full JSON context for audit deep-dives
    FOREIGN KEY(mission_id) REFERENCES mission_history(id)
);
INSERT INTO oversight_log VALUES('deea1143-8355-4a7c-8ab2-5e82bef7a5ca','4a7772a9-54a3-4716-87ea-45d3e5e846c0','weather_fetcher','tool_call','fetch_url','{"url":"https://wttr.in/19702?format=j1"}','pending',NULL,NULL,NULL,'2026-03-30 19:15:42','{"id":"32600119-1023-4437-b62d-ecd64da60568","mission_id":"4a7772a9-54a3-4716-87ea-45d3e5e846c0","agentId":"weather_fetcher","skill":"fetch_url","params":{"url":"https://wttr.in/19702?format=j1"},"department":"Swarm Core","description":"External retrieval from: https://wttr.in/19702?format=j1","timestamp":"2026-03-30T19:15:42.834271200+00:00"}');
CREATE TABLE agent_quotas (
    id TEXT PRIMARY KEY NOT NULL,
    entity_id TEXT NOT NULL UNIQUE,
    budget_usd REAL NOT NULL DEFAULT 1.0,
    used_usd REAL NOT NULL DEFAULT 0.0,
    reset_period TEXT NOT NULL CHECK(reset_period IN ('daily', 'monthly', 'never')),
    last_reset_at DATETIME NOT NULL,
    next_reset_at DATETIME NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
INSERT INTO agent_quotas VALUES('dfc21f93-b5ab-4d74-b59f-3fc9ec4b70a4','1',0.5,0.02378000000000000252,'daily','2026-04-02T23:26:29.127698200+00:00','2026-04-03T23:26:29.127698200+00:00','2026-03-30 16:25:04');
INSERT INTO agent_quotas VALUES('34412890-b161-42f4-bde9-ea6bd18d1c69','2',0.5,0.06261200000000000098,'daily','2026-03-30T16:29:29.152616+00:00','2026-03-31T16:29:29.152616+00:00','2026-03-30 16:29:29');
INSERT INTO agent_quotas VALUES('c44c7924-c700-49c3-8ad3-266b884a1d23','weather_fetcher',0.5,0.05856400000000000494,'daily','2026-03-30T19:15:40.715435300+00:00','2026-03-31T19:15:40.715435300+00:00','2026-03-30 19:15:40');
CREATE TABLE audit_trail (
    id TEXT PRIMARY KEY NOT NULL,
    agent_id TEXT NOT NULL,
    action TEXT NOT NULL,
    params TEXT NOT NULL,
    prev_hash TEXT NOT NULL,
    current_hash TEXT NOT NULL,
    timestamp DATETIME NOT NULL,
    signature TEXT,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
, mission_id TEXT, user_id TEXT);
INSERT INTO audit_trail VALUES('c911a9f1-cdff-4747-9af3-eaf85d5b22ca','1','issue_alpha_directive','{"directive":"User greeted the system. Respond with a friendly acknowledgment."}','0000000000000000000000000000000000000000000000000000000000000000','f897392094dc18d9d34a745232418b8bd506c32af6f8f2eee5b76ea92eba1f55','2026-03-30T16:29:29.151967200+00:00',NULL,'2026-03-30 16:29:29','6833bf88-c260-4769-9dec-41e94ec3135d',NULL);
INSERT INTO audit_trail VALUES('d698186a-3484-46fe-b613-a159c43c26db','1','share_finding','{"finding":"Unable to perform external GitHub searches as no tool for external web access is available. The system only supports internal codebase queries and mission knowledge searches.","topic":"ExternalSearchLimitation"}','f897392094dc18d9d34a745232418b8bd506c32af6f8f2eee5b76ea92eba1f55','e849af56c2e5af58db67fa528e3da7e5dec04b19daa397650981edaf29b8d38b','2026-03-30T19:13:20.071654200+00:00',NULL,'2026-03-30 19:13:20','2bba86ca-9826-4ab8-b0f4-9e64f06d6181',NULL);
INSERT INTO audit_trail VALUES('45617cb2-d9e5-441d-b37a-41f31f1d276a','weather_fetcher','fetch_url','{"url":"https://wttr.in/19702?format=j1"}','e849af56c2e5af58db67fa528e3da7e5dec04b19daa397650981edaf29b8d38b','3a810320260511aae3b63cc6a4d6d43014d379bf49ec0dad75b400694c7fdb0a','2026-03-30T19:15:42.834316900+00:00',NULL,'2026-03-30 19:15:42','4a7772a9-54a3-4716-87ea-45d3e5e846c0',NULL);
INSERT INTO audit_trail VALUES('c15b83a8-9542-47d3-825f-9964923fc72f','oversight','oversight_decision','{"approved":true,"entry_id":"deea1143-8355-4a7c-8ab2-5e82bef7a5ca"}','3a810320260511aae3b63cc6a4d6d43014d379bf49ec0dad75b400694c7fdb0a','6156cad70280ede4e79e712cdc09682481d388401473a017ad63b3e1063c8922','2026-03-30T19:15:51.690441900+00:00',NULL,'2026-03-30 19:15:51','4a7772a9-54a3-4716-87ea-45d3e5e846c0',NULL);
CREATE TABLE workflows (
    id              TEXT PRIMARY KEY,
    name            TEXT NOT NULL,
    description     TEXT,
    enabled         INTEGER NOT NULL DEFAULT 1,
    created_at      TEXT NOT NULL,
    updated_at      TEXT NOT NULL,
    metadata        TEXT                     -- JSON blob for overall configuration
, category TEXT DEFAULT 'user');
CREATE TABLE workflow_steps (
    id              TEXT PRIMARY KEY,
    workflow_id     TEXT NOT NULL REFERENCES workflows(id) ON DELETE CASCADE,
    agent_id        TEXT NOT NULL,           -- Agent assigned to this step
    step_order      INTEGER NOT NULL,        -- Sequence in the pipeline
    name            TEXT NOT NULL,
    prompt_template TEXT NOT NULL,           -- The prompt for this step (can use variables)
    config          TEXT,                    -- JSON configuration for this step (e.g. overrides)
    UNIQUE(workflow_id, step_order)
);
CREATE TABLE workflow_runs (
    id              TEXT PRIMARY KEY,
    workflow_id     TEXT NOT NULL REFERENCES workflows(id) ON DELETE CASCADE,
    started_at      TEXT NOT NULL,
    completed_at    TEXT,
    status          TEXT NOT NULL,           -- "running" | "completed" | "failed" | "cancelled"
    current_step    INTEGER NOT NULL DEFAULT 0,
    context         TEXT                     -- JSON blob accumulating state across steps
);
CREATE TABLE workflow_step_runs (
    id              TEXT PRIMARY KEY,
    run_id          TEXT NOT NULL REFERENCES workflow_runs(id) ON DELETE CASCADE,
    step_id         TEXT NOT NULL REFERENCES workflow_steps(id) ON DELETE CASCADE,
    mission_id      TEXT,                    -- Links to the actual agent mission
    started_at      TEXT NOT NULL,
    completed_at    TEXT,
    status          TEXT NOT NULL,           -- "completed" | "failed"
    output_text     TEXT,                    -- Captured output for the next step's context
    cost_usd        REAL NOT NULL DEFAULT 0.0
);
CREATE TABLE mission_quotas (
    id TEXT PRIMARY KEY NOT NULL,
    cluster_id TEXT NOT NULL UNIQUE,
    budget_usd REAL NOT NULL DEFAULT 5.0,
    used_usd REAL NOT NULL DEFAULT 0.0,
    reset_period TEXT NOT NULL CHECK(reset_period IN ('daily', 'monthly', 'never')),
    last_reset_at DATETIME NOT NULL,
    next_reset_at DATETIME NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE TABLE sync_manifest (
    id TEXT PRIMARY KEY,
    agent_id TEXT NOT NULL,
    source_type TEXT NOT NULL, -- 'slack', 'notion', 'fs', etc.
    source_uri TEXT NOT NULL,  -- file path, channel id, or page id
    last_sync_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    checksum TEXT,             -- Optional: for detecting file changes
    status TEXT DEFAULT 'idle', -- 'idle', 'syncing', 'error'
    metadata TEXT,             -- JSON blob for source-specific state (e.g. pagination cursors)
    FOREIGN KEY (agent_id) REFERENCES agents(id) ON DELETE CASCADE
);
CREATE INDEX idx_scheduled_jobs_next_run ON scheduled_jobs(next_run_at, enabled);
CREATE INDEX idx_scheduled_job_runs_job_id ON scheduled_job_runs(job_id);
CREATE INDEX idx_agent_quotas_entity ON agent_quotas(entity_id);
CREATE INDEX idx_audit_timestamp ON audit_trail(timestamp);
CREATE INDEX idx_audit_agent ON audit_trail(agent_id);
CREATE INDEX idx_workflow_steps_workflow_id ON workflow_steps(workflow_id);
CREATE INDEX idx_workflow_runs_workflow_id ON workflow_runs(workflow_id);
CREATE INDEX idx_workflow_step_runs_run_id ON workflow_step_runs(run_id);
CREATE INDEX idx_audit_mission ON audit_trail(mission_id);
CREATE INDEX idx_audit_user ON audit_trail(user_id);
CREATE INDEX idx_mission_quotas_cluster ON mission_quotas(cluster_id);
CREATE INDEX idx_sync_manifest_agent ON sync_manifest(agent_id);
CREATE INDEX idx_sync_manifest_status ON sync_manifest(status);
COMMIT;
