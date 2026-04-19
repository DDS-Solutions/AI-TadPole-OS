# Incident Response Protocol

1. Immediate sidecar process kill (Taskkill /F).
2. Wipe 'tadpole.db-shm' and '-wal' locks.
3. Check port 8000 availability.
4. Restart Engine in 'SAFE-MODE'.
5. Broadcast emergency status to 'Sovereign'.
6. Run POST-MORTEM analysis.
