# mission_dual_node.ps1
# Triggers a hierarchical mission: Tadpole (COO) -> Elon (CTO)

$AgentId = "2" # Tadpole (COO)
$Url = "http://localhost:8000/agents/$AgentId/send"

$Body = @{
    message = "MISSION START: System-Wide Security Integration Audit.
    1. Recruit Elon (ID: 3) to analyze the 'server-rs' source code for any hardcoded secrets or unsafe blocks.
    2. Once Elon provides the technical breakdown, synthesize his findings into a high-level Executive Risk Report.
    3. Ensure the final report is archived to the vault.
    Explain your coordination steps for the dashboard log."
} | ConvertTo-Json

$Headers = @{
    "Authorization" = "Bearer tadpole-dev-token-2026"
    "Content-Type"  = "application/json"
}

Write-Host "🚀 Dispatching Dual-Node Mission to Tadpole ($AgentId)..."
try {
    $Response = Invoke-RestMethod -Uri $Url -Method Post -Body $Body -Headers $Headers
    Write-Host "✅ Mission Dispatched."
    Write-Host "Response: $($Response | ConvertTo-Json -Depth 2)"
    Write-Host "`n👀 Observe the CO-OP between Tadpole and Elon on the Dashboard."
}
catch {
    Write-Host "❌ Error: $_"
}
