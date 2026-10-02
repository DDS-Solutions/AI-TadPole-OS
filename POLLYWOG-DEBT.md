# 📋 Tadpole OS: Technical Debt Ledger

> **Status**: Stub (Round-3 docs fix, 2026-10-01)  
> **Tip**: `bbcf0d4` / v1.1.463

The previous auto-generated ledger was **corrupted** (embedded `file:///D:\...` paths and Python `sys.exit` fragments). Do not treat that content as authoritative.

## Where to look instead

| Source | Use |
| :--- | :--- |
| [`ROADMAP.md`](ROADMAP.md) | Planned phases and feature intent |
| [`TODO.md`](TODO.md) | Open backlog items |
| `pollywog:` comments in source | Inline ceilings (regenerate ledger via `execution/pollywog_debt_ledger.py` when the generator is healthy) |

## Regenerating

When ready to restore an automated ledger:

```bash
python execution/pollywog_debt_ledger.py
```

Verify the output table has real file paths (repo-relative) and no embedded interpreter source before committing.
