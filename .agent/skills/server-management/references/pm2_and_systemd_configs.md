> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / server-management
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Systemd crash loops or log disk exhaustion.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SERVER_MANAGEMENT]`)

# Production Process Management & Server Configs (L3)

---

## 1. Systemd Service Unit (`/etc/systemd/system/tadpole-server.service`)

```ini
[Unit]
Description=Tadpole OS Sovereign Backend Engine
After=network.target

[Service]
Type=simple
User=tadpole
WorkingDirectory=/opt/tadpole-os
ExecStart=/opt/tadpole-os/target/release/server
Restart=always
RestartSec=5s
Environment=PORT=8000
Environment=TADPOLE_ENV=production
LimitNOFILE=65535

[Install]
WantedBy=multi-user.target
```

---

## 2. PM2 Ecosystem Configuration (`ecosystem.config.js`)

```javascript
module.exports = {
  apps: [
    {
      name: 'tadpole-gateway',
      script: 'npm',
      args: 'run start',
      instances: 'max',
      exec_mode: 'cluster',
      autorestart: true,
      max_memory_restart: '1G',
      env_production: {
        NODE_ENV: 'production'
      }
    }
  ]
};
```