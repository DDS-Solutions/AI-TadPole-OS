-- Convert existing agent_quotas to micro-dollars if they are in standard float representation
UPDATE agent_quotas 
SET budget_usd = CAST(ROUND(budget_usd * 1000000) AS INTEGER), 
    used_usd = CAST(ROUND(used_usd * 1000000) AS INTEGER)
WHERE budget_usd < 100000;

-- Convert existing mission_quotas to micro-dollars if they are in standard float representation
UPDATE mission_quotas 
SET budget_usd = CAST(ROUND(budget_usd * 1000000) AS INTEGER), 
    used_usd = CAST(ROUND(used_usd * 1000000) AS INTEGER)
WHERE budget_usd < 100000;
