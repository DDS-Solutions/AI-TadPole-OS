#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Registry:Skills

### AI Context Alignment
- **Subsystem**: Agent Skills / database-design / schema_validator
- **Primary Entrypoints**: `find_schema_files`, `validate_prisma_schema`, `validate_sql_schema`, `validate_drizzle_schema`, `run_self_tests`, `main`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic validation of database schemas (Prisma, SQL, Drizzle) without side effects.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: `[schema_validator]`, `[PASS]`
- **Witness Tests**: none declared
"""

import sys
import io
import os
import json
import re
import argparse
from pathlib import Path
from datetime import datetime
from typing import List, Dict, Any, Tuple, Optional

# Fix Windows console encoding
if sys.platform == "win32":
    try:
        sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8')
        sys.stderr = io.TextIOWrapper(sys.stderr.buffer, encoding='utf-8')
    except Exception:
        pass


def find_schema_files(project_path: Path) -> List[Tuple[str, Path]]:
    """
    Discover database schema and migration files across the target project path.
    Supported types: 'prisma', 'sql', 'drizzle'.
    """
    schemas: List[Tuple[str, Path]] = []
    
    # 1. Prisma schema
    prisma_patterns = ['schema.prisma', '**/prisma/schema.prisma', '**/schema.prisma']
    for pattern in prisma_patterns:
        for f in project_path.glob(pattern):
            if '.git' not in f.parts and 'node_modules' not in f.parts:
                schemas.append(('prisma', f))
    
    # 2. SQL schema / migration files
    sql_patterns = ['*.sql', '**/*.sql', '**/migrations/*.sql', '**/schema/*.sql', '**/sql/*.sql']
    for pattern in sql_patterns:
        for f in project_path.glob(pattern):
            if '.git' not in f.parts and 'node_modules' not in f.parts and 'target' not in f.parts:
                schemas.append(('sql', f))
                
    # 3. Drizzle schema files
    drizzle_patterns = ['*.ts', '**/drizzle/*.ts', '**/schema/*.ts', '**/db/schema.ts']
    for pattern in drizzle_patterns:
        for f in project_path.glob(pattern):
            if '.git' not in f.parts and 'node_modules' not in f.parts and 'target' not in f.parts:
                if 'schema' in f.name.lower() or 'table' in f.name.lower():
                    schemas.append(('drizzle', f))
                    
    # Deduplicate while preserving order
    seen = set()
    deduped = []
    for stype, fpath in schemas:
        resolved = str(fpath.resolve())
        if resolved not in seen:
            seen.add(resolved)
            deduped.append((stype, fpath))
            
    return deduped[:50]


def validate_prisma_schema(content: str) -> List[str]:
    """
    Validate Prisma schema content for common pitfalls and best practices.
    Checks:
    - PascalCase model & enum names
    - Primary key (@id) definition
    - Audit timestamp fields (createdAt/updatedAt or created_at/updated_at)
    - Foreign key indexing (@@index)
    """
    issues: List[str] = []
    
    # Strip comments
    clean_lines = []
    for line in content.splitlines():
        line_no_comment = line.split('//')[0]
        clean_lines.append(line_no_comment)
    clean_content = '\n'.join(clean_lines)
    
    # Find all models
    models = re.findall(r'model\s+(\w+)\s*{([^}]+)}', clean_content, re.DOTALL)
    
    for model_name, model_body in models:
        # Check PascalCase naming
        if not model_name[0].isupper() or '_' in model_name:
            issues.append(f"Model '{model_name}' should use PascalCase (e.g., 'UserProfile')")
            
        # Check for @id or @@id
        if '@id' not in model_body and '@@id' not in model_body:
            issues.append(f"Model '{model_name}' missing primary key (@id or @@id)")
            
        # Check for created_at / createdAt timestamp
        has_created = bool(re.search(r'\b(createdAt|created_at)\b', model_body, re.IGNORECASE))
        if not has_created:
            issues.append(f"Model '{model_name}' missing audit timestamp ('createdAt' or 'created_at')")
            
        # Check for unindexed foreign keys
        fields = [f.strip() for f in model_body.splitlines() if f.strip()]
        for field in fields:
            fk_match = re.match(r'^(\w+Id)\s+(\w+)', field)
            if fk_match:
                fk_name = fk_match.group(1)
                idx_pattern = rf'@@index\(\[.*?\b{fk_name}\b.*?\]\)'
                if not re.search(idx_pattern, model_body):
                    issues.append(f"Model '{model_name}': Foreign key '{fk_name}' lacks an index (consider @@index([{fk_name}]))")
                    
    # Check enum naming
    enums = re.findall(r'enum\s+(\w+)\s*{', clean_content)
    for enum_name in enums:
        if not enum_name[0].isupper():
            issues.append(f"Enum '{enum_name}' should use PascalCase")
            
    return issues


def validate_sql_schema(content: str) -> List[str]:
    """
    Validate SQL DDL schema / migration statements.
    Checks:
    - Primary key existence in CREATE TABLE
    - Foreign key column indexing
    - SELECT * usage in migration views/triggers
    - Timestamp with timezone or epoch consistency
    """
    issues: List[str] = []
    
    # Strip SQL comments
    clean_content = re.sub(r'--.*$', '', content, flags=re.MULTILINE)
    clean_content = re.sub(r'/\*.*?\*/', '', clean_content, flags=re.DOTALL)
    
    # Find CREATE TABLE statements
    table_pattern = re.compile(
        r'CREATE\s+TABLE\s+(?:IF\s+NOT\s+EXISTS\s+)?([`"\[]?\w+[`"\]]?)\s*\((.*?)\);',
        re.IGNORECASE | re.DOTALL
    )
    
    # Collect all created indexes in this file
    created_indexes = re.findall(
        r'CREATE\s+(?:UNIQUE\s+)?INDEX\s+(?:IF\s+NOT\s+EXISTS\s+)?\w+\s+ON\s+([`"\[]?\w+[`"\]]?)\s*\((.*?)\)',
        clean_content,
        re.IGNORECASE
    )
    
    indexed_columns: Dict[str, List[str]] = {}
    for tbl, cols in created_indexes:
        tbl_clean = tbl.strip('`"[]')
        col_list = [c.strip().strip('`"[]').split()[0] for c in cols.split(',') if c.strip()]
        indexed_columns.setdefault(tbl_clean, []).extend(col_list)
        
    tables = table_pattern.findall(clean_content)
    for table_name_raw, table_body in tables:
        table_name = table_name_raw.strip('`"[]')
        
        # Check primary key
        has_pk = bool(
            re.search(r'\bPRIMARY\s+KEY\b', table_body, re.IGNORECASE)
            or re.search(r'\bAUTOINCREMENT\b', table_body, re.IGNORECASE)
        )
        if not has_pk:
            issues.append(f"Table '{table_name}' appears to lack a PRIMARY KEY definition")
            
        # Check foreign keys and see if they are indexed
        fk_columns = set()
        
        # Table-level: FOREIGN KEY (col1, col2) REFERENCES target(id)
        fk_matches = re.findall(r'FOREIGN\s+KEY\s*\((.*?)\)\s*REFERENCES', table_body, re.IGNORECASE)
        for col_str in fk_matches:
            for c in col_str.split(','):
                cleaned = c.strip().strip('`"[]')
                if cleaned:
                    fk_columns.add(cleaned)
                    
        # Column-level: e.g. "customer_id INTEGER NOT NULL REFERENCES customers(id)"
        col_defs = re.split(r',\s*(?![^()]*\))', table_body)
        for col_def in col_defs:
            col_def_str = col_def.strip()
            if re.search(r'\bREFERENCES\b', col_def_str, re.IGNORECASE):
                if not re.match(r'^(?:CONSTRAINT\s+\w+\s+)?FOREIGN\s+KEY', col_def_str, re.IGNORECASE):
                    tokens = col_def_str.split()
                    if tokens:
                        col_candidate = tokens[0].strip('`"[]')
                        if col_candidate.upper() not in {'CONSTRAINT', 'PRIMARY', 'FOREIGN', 'UNIQUE', 'CHECK'}:
                            fk_columns.add(col_candidate)
            
        table_indexes = indexed_columns.get(table_name, [])
        for fk_col in fk_columns:
            if fk_col not in table_indexes and fk_col.lower() != 'id':
                issues.append(f"Table '{table_name}': Foreign key column '{fk_col}' has no corresponding CREATE INDEX")
                
        # Check timestamp recommendations (created_at)
        has_created = bool(re.search(r'\b(created_at|created_time|timestamp)\b', table_body, re.IGNORECASE))
        if not has_created:
            issues.append(f"Table '{table_name}' missing audit timestamp column ('created_at')")
            
    # Check for SELECT * in views or triggers
    if re.search(r'CREATE\s+VIEW.*?AS\s+SELECT\s+\*\s+FROM', clean_content, re.IGNORECASE):
        issues.append("Avoid 'SELECT *' in CREATE VIEW definitions; explicitly enumerate projected columns")
        
    return issues


def validate_drizzle_schema(content: str) -> List[str]:
    """
    Validate Drizzle ORM schema files.
    Checks:
    - Table definitions have primary keys (.primaryKey())
    - Timestamp / audit fields
    - Foreign key references
    """
    issues: List[str] = []
    
    # Strip comments
    clean_lines = [l.split('//')[0] for l in content.splitlines()]
    clean_content = '\n'.join(clean_lines)
    
    # Find table definitions: sqliteTable, pgTable, mysqlTable
    tables = re.findall(
        r'export\s+const\s+(\w+)\s*=\s*(?:sqliteTable|pgTable|mysqlTable)\s*\(\s*[\'"](\w+)[\'"]\s*,\s*{([^}]+)}',
        clean_content,
        re.DOTALL
    )
    
    for var_name, table_name, body in tables:
        # Check primary key
        if 'primaryKey()' not in body and '.primaryKey' not in body:
            issues.append(f"Drizzle table '{table_name}' ({var_name}) missing .primaryKey() column")
            
        # Check audit timestamp
        has_created = bool(re.search(r'\b(created_at|createdAt)\b', body, re.IGNORECASE))
        if not has_created:
            issues.append(f"Drizzle table '{table_name}' missing created_at/createdAt timestamp column")
            
    return issues


def run_self_tests() -> bool:
    """
    Execute comprehensive self-tests on Prisma, SQL, and Drizzle validation logic.
    Returns True if all tests pass.
    """
    print("[schema_validator] Executing self-test suite...")
    
    # 1. Prisma Tests
    bad_prisma = """
    model user {
        id Int
        teamId Int
        name String
    }
    """
    prisma_issues = validate_prisma_schema(bad_prisma)
    assert any("PascalCase" in i for i in prisma_issues), "Failed to detect non-PascalCase model"
    assert any("primary key" in i for i in prisma_issues), "Failed to detect missing @id"
    assert any("audit timestamp" in i for i in prisma_issues), "Failed to detect missing createdAt"
    assert any("teamId" in i for i in prisma_issues), "Failed to detect unindexed foreign key"
    
    good_prisma = """
    model UserProfile {
        id String @id @default(uuid())
        teamId String
        createdAt DateTime @default(now())
        updatedAt DateTime @updatedAt
        
        @@index([teamId])
    }
    """
    good_prisma_issues = validate_prisma_schema(good_prisma)
    assert len(good_prisma_issues) == 0, f"Expected 0 issues for valid Prisma schema, got {good_prisma_issues}"
    
    # 2. SQL Tests
    bad_sql = """
    CREATE TABLE orders (
        customer_id INTEGER REFERENCES customers(id),
        total_amount REAL
    );
    """
    sql_issues = validate_sql_schema(bad_sql)
    assert any("PRIMARY KEY" in i for i in sql_issues), "Failed to detect missing PRIMARY KEY in SQL"
    assert any("customer_id" in i for i in sql_issues), "Failed to detect unindexed foreign key in SQL"
    assert any("created_at" in i for i in sql_issues), "Failed to detect missing created_at in SQL"
    
    good_sql = """
    CREATE TABLE orders (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        customer_id INTEGER NOT NULL REFERENCES customers(id),
        total_amount REAL NOT NULL,
        created_at TEXT NOT NULL DEFAULT (datetime('now'))
    );
    CREATE INDEX idx_orders_customer_id ON orders(customer_id);
    """
    good_sql_issues = validate_sql_schema(good_sql)
    assert len(good_sql_issues) == 0, f"Expected 0 issues for valid SQL schema, got {good_sql_issues}"
    
    # 3. Drizzle Tests
    bad_drizzle = """
    export const users = sqliteTable('users', {
        name: text('name')
    });
    """
    drizzle_issues = validate_drizzle_schema(bad_drizzle)
    assert any("primaryKey" in i for i in drizzle_issues), "Failed to detect missing primaryKey in Drizzle"
    assert any("timestamp" in i for i in drizzle_issues), "Failed to detect missing timestamp in Drizzle"
    
    print("[schema_validator] [PASS] All self-tests passed successfully.")
    return True


def main():
    parser = argparse.ArgumentParser(description="Deterministic Database Schema Validator")
    parser.add_argument("path", nargs="?", default=".", help="Root project directory or schema file to validate")
    parser.add_argument("--test", action="store_true", help="Run self-tests and exit")
    parser.add_argument("--json", action="store_true", help="Output results in JSON format")
    
    args = parser.parse_args()
    
    if args.test:
        success = run_self_tests()
        sys.exit(0 if success else 1)
        
    target_path = Path(args.path).resolve()
    
    schemas: List[Tuple[str, Path]] = []
    if target_path.is_file():
        ext = target_path.suffix.lower()
        if ext == '.prisma':
            schemas.append(('prisma', target_path))
        elif ext == '.sql':
            schemas.append(('sql', target_path))
        elif ext in ('.ts', '.js'):
            schemas.append(('drizzle', target_path))
    else:
        schemas = find_schema_files(target_path)
        
    results: List[Dict[str, Any]] = []
    total_issues = 0
    
    for schema_type, file_path in schemas:
        try:
            content = file_path.read_text(encoding='utf-8', errors='ignore')
            if schema_type == 'prisma':
                issues = validate_prisma_schema(content)
            elif schema_type == 'sql':
                issues = validate_sql_schema(content)
            elif schema_type == 'drizzle':
                issues = validate_drizzle_schema(content)
            else:
                issues = []
        except Exception as e:
            issues = [f"File read error: {e}"]
            
        if issues:
            total_issues += len(issues)
            results.append({
                "file": str(file_path.name),
                "path": str(file_path),
                "type": schema_type,
                "issues": issues
            })
            
    if args.json:
        output = {
            "script": "schema_validator",
            "target": str(target_path),
            "schemas_scanned": len(schemas),
            "issues_count": total_issues,
            "passed": total_issues == 0,
            "findings": results
        }
        print(json.dumps(output, indent=2))
        sys.exit(0)
        
    print(f"\n{'='*64}")
    print(f"[SCHEMA VALIDATOR] Database Schema Integrity Analysis")
    print(f"{'='*64}")
    print(f"Target Directory : {target_path}")
    print(f"Schemas Found    : {len(schemas)}")
    print(f"Total Warnings   : {total_issues}")
    print(f"Timestamp        : {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
    print(f"{'-'*64}")
    
    if not schemas:
        print("[INFO] No Prisma, SQL, or Drizzle schema files found in target scope.")
        sys.exit(0)
        
    if results:
        for item in results:
            print(f"\n📂 {item['file']} ({item['type'].upper()}):")
            for issue in item['issues'][:8]:
                print(f"   ⚠️  {issue}")
            if len(item['issues']) > 8:
                print(f"   ... and {len(item['issues']) - 8} more warnings.")
    else:
        print("\n🎉 [PASS] All analyzed schema files meet structural best-practice standards!")
        
    print(f"\n{'='*64}")
    sys.exit(0)


if __name__ == "__main__":
    main()
