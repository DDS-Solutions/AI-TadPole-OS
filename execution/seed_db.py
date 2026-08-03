"""
@docs ARCHITECTURE:Infrastructure:Execution

### AI Assist Note
**Database Seeding Engine**: Populates tadpole.db with high-fidelity historical data.
Provides a mock telemetry base for generating performance delta and metrics reports.

### 🔍 Debugging & Observability
- **Failure Path**: sqlite3.Error when executing seeding queries.
- **Telemetry Link**: Search `[seed_db]` in system logs.
"""

import sqlite3
import os
import uuid
from datetime import datetime, timedelta, timezone

def seed_database(db_path: str, force: bool = False):
    print(f"[*] [seed_db] Starting database seed on: {db_path}")
    if not os.path.exists(db_path):
        print(f"[FAIL] Database file not found at {db_path}")
        return

    try:
        conn = sqlite3.connect(db_path)
        cursor = conn.cursor()

        # Check if already seeded (e.g., if there are already missions and quotas)
        cursor.execute("SELECT COUNT(*) FROM mission_history;")
        existing_missions = cursor.fetchone()[0]

        if existing_missions > 50 and not force:
            print(f"[!] Database already populated with {existing_missions} missions. Skipping seed.")
            conn.close()
            return

        # 1. Seed Quotas
        print("[*] Seeding agent quotas...")
        mock_quotas = [
            ('q-coder', 'coder', 10.0, 4.25, 'daily', '2026-07-13T00:00:00Z', '2026-07-14T00:00:00Z'),
            ('q-marketing', 'marketing', 5.0, 1.12, 'daily', '2026-07-13T00:00:00Z', '2026-07-14T00:00:00Z'),
            ('q-security', 'security', 20.0, 0.0, 'daily', '2026-07-13T00:00:00Z', '2026-07-14T00:00:00Z'),
            ('q-researcher', 'researcher', 15.0, 8.50, 'daily', '2026-07-13T00:00:00Z', '2026-07-14T00:00:00Z'),
            ('q-governor', 'governor', 50.0, 12.40, 'monthly', '2026-07-01T00:00:00Z', '2026-08-01T00:00:00Z')
        ]
        
        for q in mock_quotas:
            cursor.execute(
                "INSERT OR REPLACE INTO agent_quotas (id, entity_id, budget_usd, used_usd, reset_period, last_reset_at, next_reset_at) VALUES (?, ?, ?, ?, ?, ?, ?);",
                q
            )

        # 2. Seed Mission History (Spanning last 14 days)
        print("[*] Seeding mission history...")
        agents = ['coder', 'marketing', 'security', 'researcher', 'governor']
        statuses = ['completed', 'completed', 'completed', 'failed', 'completed']
        
        now = datetime.now(timezone.utc)
        for i in range(40):
            m_id = str(uuid.uuid4())
            agent = agents[i % len(agents)]
            status = statuses[i % len(statuses)]
            delta_days = (i * 8) // 24  # spread over last 14 days
            created_time = now - timedelta(days=delta_days, hours=i % 24)
            updated_time = created_time + timedelta(minutes=15)
            
            cost = round((i * 0.12) % 1.5, 3)
            budget = 2.0
            
            cursor.execute(
                "INSERT INTO mission_history (id, agent_id, title, status, created_at, updated_at, budget_usd, cost_usd, is_degraded, is_pinned) VALUES (?, ?, ?, ?, ?, ?, ?, ?, 0, 0);",
                (m_id, agent, f"Telemetry Verification Run #{i}", status, created_time.isoformat(), updated_time.isoformat(), budget, cost)
            )

            # Insert logs for completed/failed runs
            cursor.execute(
                "INSERT INTO mission_logs (id, mission_id, agent_id, source, text, severity, timestamp) VALUES (?, ?, ?, ?, ?, ?, ?);",
                (str(uuid.uuid4()), m_id, agent, 'System', f"Initializing telemetry link for mission {m_id}", 'info', created_time.isoformat())
            )
            
            if status == 'completed':
                cursor.execute(
                    "INSERT INTO mission_logs (id, mission_id, agent_id, source, text, severity, timestamp) VALUES (?, ?, ?, ?, ?, ?, ?);",
                    (str(uuid.uuid4()), m_id, agent, 'System', "Telemetry loop executed successfully. Results verified.", 'info', updated_time.isoformat())
                )
            else:
                cursor.execute(
                    "INSERT INTO mission_logs (id, mission_id, agent_id, source, text, severity, timestamp) VALUES (?, ?, ?, ?, ?, ?, ?);",
                    (str(uuid.uuid4()), m_id, agent, 'System', "FATAL: LLM provider rate limit reached (HTTP 429). Exiting loop.", 'error', updated_time.isoformat())
                )

        conn.commit()
        print("[OK] Seeding completed successfully.")
        conn.close()
    except Exception as e:
        print(f"[FAIL] Error seeding database: {e}")

if __name__ == "__main__":
    db_path = r"D:\TadpoleOS-Dev\data\tadpole.db"
    seed_database(db_path, force=True)
