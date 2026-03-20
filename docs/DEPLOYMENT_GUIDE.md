# 🚢 Tadpole OS: Deployment Guide (Swarm Bunker)

This guide details the procedure for deploying and hosting the **Tadpole OS Engine** on a Swarm Bunker from a Windows development machine.

## 📋 System Requirements

### Host (Windows Dev Machine)
- **PowerShell 7+**
- **OpenSSH Client** (integrated with Windows)
- **tar.exe** (integrated with Windows)

### Target (Swarm Bunker (Linux or Similar Sandbox for Production Deployment))
- **Docker & Docker Compose**
- **Tailscale** (Recommended for secure, zero-config networking)
- **SSH Server** (Configured for passwordless auth)

---

## 🔐 1. Security & SSH Setup

To enable the automated `deploy.ps1` script, you must configure passwordless SSH access.

1. **Generate Key Pair** (on Windows):
   ```powershell
   ssh-keygen -t ed25519
   ```
2. **Transfer Public Key**:
   Copy the contents of `~/.ssh/id_ed25519.pub` to the target's `~/.ssh/authorized_keys`.
3. **Configure SSH Aliases**:
   Add entries for your bunkers in `C:\Users\<User>\.ssh\config`:
   ```config
   # Swarm Bunker 1
   Host tadpole-linux
       HostName 192.168.50.15
       User home office_pc
       IdentityFile ~/.ssh/tadpole_deploy_key

   # Swarm Bunker 2 (High RAM)
   Host tadpole-bunker-2
       HostName 192.168.50.38
       User linuxlite66
       IdentityFile ~/.ssh/tadpole_deploy_key
   ```

---

## 🚀 2. The Deployment Pipeline

The `deploy.ps1` script automates the entire distribution lifecycle:

### Execution
Run the specific script for your target bunker:
```powershell
# Deploy to Bunker 1 (Standard)
./deploy-bunker-1.ps1

# Deploy to Bunker 2 (High RAM)
./deploy-bunker-2.ps1
```

### What happens under the hood:
1.  **Packaging**: The script uses `tar` to create a lightweight deployment bundle, excluding heavy directories like `node_modules` and `target`.
2.  **Transfer**: The bundle is streamed to the Linux host via `scp`.
3.  **Rollback Protection**: 
    - The current running image is tagged as `tadpole-os:rollback`.
    - If the new build fails health checks, the operator can instantly revert by running `docker tag tadpole-os:rollback tadpole-os:latest && docker compose up -d`.
4.  **Remote Build**: The script SSHs into the target host, extracts the bundle, and triggers `docker compose up --build -d` to produce a fresh production image.

### Monitoring & Maintenance

- **Health Checks**: The engine exposes `/engine/health`. Monitor this via Docker or specialized agents.
- **Persistent Data**: Ensure your Docker Compose configuration maps a host volume to the container's `/data/swarm_config/` directory. This is critical so that any agent swarms downloaded from the **Template Store** survive container restarts and deployments.
- **Post-Deployment Verification**: Run a standard "Smoke Test" mission with **Mission Analysis** enabled. This ensures that the newly deployed engine correctly integrates with providers and produces valid success reports.
- **Agent Capacity**: The current sector is configured for **25 default agents**. This threshold is configurable. For massive swarms, increase the `MAX_NODES` constant in the frontend and ensure your provider's rate limits accommodate the increased concurrency of **Parallel Swarming**.
- **Log Management**: Engine logs are written to internal Docker storage. Use `docker logs tadpole-os -f` for real-time telemetry.

### Scale-up Strategy (Horizontal Expansion)
To support more than 25 agents, follow this multi-sector deployment pattern:
1. **Provision New Node**: Deploy a second instance of the Tadpole OS Engine on a separate host/port.
2. **Cluster Link**: Update the frontend `settingsStore.ts` or environment variables to point to the new sector.
3. **Database Sync**: The foundational SQLite database (`tadpole.db`) can be shared across sectors via a network volume (e.g. NFS) if unified state is required.

---

## 🛠️ 3. Environment Configuration

Your secrets and configurations are managed via a `.env` file on the remote host (or passed via the CI/CD environment).

**Required Variables (`.env`):**
```env
NEURAL_TOKEN=your-secure-token
MERKLE_AUDIT_ENABLED=true
RESOURCE_GUARD_ENABLED=true
SANDBOX_AWARENESS=true
GOOGLE_API_KEY=AIza...
GROQ_API_KEY=gsk_...
ALLOWED_ORIGINS=http://your-dashboard-url
```

---

## 📈 4. Monitoring & Troubleshooting

### Viewing Remote Logs
To monitor the live engine on the Linux host:
```bash
ssh tadpole-linux "cd ~/Desktop/tadpole-os && docker compose logs -f"
```

### Manual Restart
```bash
ssh tadpole-linux "cd ~/Desktop/tadpole-os && docker compose restart"
```

### Accessing the Dashboard
In **Production (Docker)**, the dashboard is served directly by the Axum engine. Access it via:
`http://<bunker-ip>:8000`

> [!IMPORTANT]
> The engine binds to `0.0.0.0:8000` within the container. You do NOT need port `5173` on the target host; that is for local development only. Ensure your firewall allows traffic on port `8000`.
