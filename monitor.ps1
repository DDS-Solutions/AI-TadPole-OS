param(
    [int]$Interval = 30,
    [string]$BaseUrl = "http://localhost:8000",
    [string]$Token = "tadpole-2026-dev"
)

$headers = @{"Authorization" = "Bearer $Token"}

function Get-EngineHealth {
    try {
        $resp = Invoke-RestMethod -Uri "$BaseUrl/v1/engine/health" -Method GET -Headers $headers -ErrorAction Stop
        return $resp
    }
    catch {
        return @{status="ERROR"; error=$_.Exception.Message}
    }
}

function Get-EngineMetrics {
    try {
        $resp = Invoke-RestMethod -Uri "$BaseUrl/v1/engine/metrics" -Method GET -Headers $headers -ErrorAction Stop
        return $resp
    }
    catch {
        return "ERROR: $($_.Exception.Message)"
    }
}

function Get-AgentHealth {
    try {
        $resp = Invoke-RestMethod -Uri "$BaseUrl/v1/oversight/security/health" -Method GET -Headers $headers -ErrorAction Stop
        return $resp
    }
    catch {
        return @{error=$_.Exception.Message}
    }
}

function Get-AgentList {
    try {
        $resp = Invoke-RestMethod -Uri "$BaseUrl/v1/agents" -Method GET -Headers $headers -ErrorAction Stop
        return $resp
    }
    catch {
        return @()
    }
}

function Get-PendingOversight {
    try {
        $resp = Invoke-RestMethod -Uri "$BaseUrl/v1/oversight/pending" -Method GET -Headers $headers -ErrorAction Stop
        return $resp
    }
    catch {
        return @()
    }
}

function Get-Policies {
    try {
        $resp = Invoke-RestMethod -Uri "$BaseUrl/v1/oversight/security/policies" -Method GET -Headers $headers -ErrorAction Stop
        return $resp
    }
    catch {
        return @()
    }
}

function Write-Header {
    param([string]$Title)
    $timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
    Write-Host "`n═══════════════════════════════════════════════════════════════" -ForegroundColor Cyan
    Write-Host "  $Title  |  $timestamp" -ForegroundColor Cyan
    Write-Host "═══════════════════════════════════════════════════════════════" -ForegroundColor Cyan
}

function Format-HealthState {
    param([string]$State)
    switch ($State) {
        "Ready" { return "READY" }
        "Warming" { return "WARMING" }
        "Degraded" { return "DEGRADED" }
        default { return $State }
    }
}

Write-Host "Starting Tadpole OS Monitor (interval: ${Interval}s, Ctrl+C to stop)" -ForegroundColor Green

while ($true) {
    $health = Get-EngineHealth
    $metrics = Get-EngineMetrics
    $agentHealth = Get-AgentHealth
    $pending = Get-PendingOversight
    $policies = Get-Policies
    
    Write-Header "ENGINE HEALTH"
    $stateColor = if ($health.health_state -eq "Ready") { "Green" } elseif ($health.health_state -eq "Warming") { "Yellow" } else { "Red" }
    Write-Host "  Status:       $($health.status)" -ForegroundColor $stateColor
    Write-Host "  Health State: $($health.health_state)" -ForegroundColor $stateColor
    Write-Host "  Version:      $($health.version)"
    Write-Host "  Heartbeat:    $($health.heartbeat)"
    Write-Host "  Active Agents: $($health.active_agents)"
    Write-Host "  Features:     $($health.features -join ', ')"
    
    Write-Header "ENGINE METRICS"
    if ($metrics -is [string]) {
        $lines = $metrics -split "`n"
        foreach ($line in $lines) {
            if ($line -match '^# HELP (.+)') {
                $help = $matches[1]
                Write-Host "  $help" -ForegroundColor Gray
            } elseif ($line -match '^tadpole_(\w+)\s+(\d+)') {
                Write-Host "  $($matches[1]): $($matches[2])" -ForegroundColor White
            }
        }
    } else {
        $metrics | ForEach-Object { Write-Host "  $($_.Key): $($_.Value)" }
    }
    
    Write-Header "AGENT HEALTH"
    if ($agentHealth.agents) {
        $healthy = ($agentHealth.agents | Where-Object { $_.is_healthy -eq $true }).Count
        $unhealthy = ($agentHealth.agents | Where-Object { $_.is_healthy -eq $false }).Count
        $bankrupt = ($agentHealth.agents | Where-Object { $_.is_bankrupt -eq $true }).Count
        $throttled = ($agentHealth.agents | Where-Object { $_.is_throttled -eq $true }).Count
        $failures = ($agentHealth.agents | Where-Object { $_.failure_count -gt 0 }).Count
        
        Write-Host "  Total:       $($agentHealth.agents.Count)" -ForegroundColor White
        Write-Host "  Healthy:     $healthy" -ForegroundColor Green
        Write-Host "  Unhealthy:   $unhealthy" -ForegroundColor Red
        Write-Host "  Bankrupt:    $bankrupt" -ForegroundColor Red
        Write-Host "  Throttled:   $throttled" -ForegroundColor Yellow
        Write-Host "  w/ Failures: $failures" -ForegroundColor Yellow
        
        # Show agents with issues
        $issues = $agentHealth.agents | Where-Object { $_.is_healthy -eq $false -or $_.is_bankrupt -eq $true -or $_.is_throttled -eq $true -or $_.failure_count -gt 0 }
        if ($issues.Count -gt 0) {
            Write-Host "`n  Agents with issues:" -ForegroundColor Yellow
            $issues | Format-Table agent_id, name, is_healthy, is_bankrupt, is_throttled, failure_count -AutoSize | Out-Host
        }
    } else {
        Write-Host "  No agent health data" -ForegroundColor Gray
    }
    
    Write-Header "OVERSIGHT PENDING"
    if ($pending.Count -gt 0) {
        $pending | Format-Table id, type, status, created_at -AutoSize | Out-Host
    } else {
        Write-Host "  None" -ForegroundColor Green
    }
    
    Write-Header "SECURITY POLICIES"
    if ($policies.Count -gt 0) {
        $policies | Format-Table mode, tool_name -AutoSize | Out-Host
    } else {
        Write-Host "  None" -ForegroundColor Gray
    }
    
    Start-Sleep -Seconds $Interval
}
