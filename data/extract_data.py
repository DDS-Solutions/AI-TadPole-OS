
import sys
import re

# We only want to restore user data, not schema or migration history.
ALLOWED_TABLES = [
    'agents', 
    'mission_history', 
    'mission_logs', 
    'swarm_context', 
    'agent_quotas', 
    'audit_trail', 
    'workflows', 
    'workflow_steps'
]

def clean_dump(input_path, output_path):
    with open(input_path, 'r', encoding='utf-8') as f:
        lines = f.readlines()

    recovered_sql = [
        "PRAGMA foreign_keys=OFF;",
        "BEGIN TRANSACTION;"
    ]

    for line in lines:
        if line.startswith("INSERT INTO"):
            # Check if it's one of our allowed data tables
            # Matches: INSERT INTO table_name VALUES...
            match = re.match(r"INSERT INTO ([a-zA-Z0-9_]+)", line)
            if match:
                table_name = match.group(1)
                if table_name in ALLOWED_TABLES:
                    # Special Case: Before inserting into 'agents', 
                    # we should delete the default Alpha agent seeded by the engine.
                    if table_name == 'agents':
                        recovered_sql.append("DELETE FROM agents;")
                    
                    recovered_sql.append(line.strip())

    recovered_sql.append("COMMIT;")

    with open(output_path, 'w', encoding='utf-8') as f:
        f.write("\n".join(recovered_sql))

if __name__ == "__main__":
    clean_dump(sys.argv[1], sys.argv[2])
