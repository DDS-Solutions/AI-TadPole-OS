
# trigger_agent.ps1
# Triggers the Tadpole OS Agent via the REST API

$Url = "http://localhost:8000/agents/8f564fe4-fda6-4bd5-938b-6b02dd8d2e8f/send"
$Body = @{
    message = "Please list the files in the current directory and then create a new file called 'hello_aletheia.txt' with the content 'Thinking before acting!'."
} | ConvertTo-Json

Write-Host "🚀 Triggering Agent at $Url..."
try {
    $Response = Invoke-RestMethod -Uri $Url -Method Post -Body $Body -ContentType "application/json"
    Write-Host "✅ Success! Agent Triggered."
    Write-Host "Response: $($Response | ConvertTo-Json -Depth 2)"
    Write-Host "`n👀 Check the 'System Log' in your Dashboard to see Aletheia thinking!"
} catch {
    Write-Host "❌ Error: $_"
    Write-Host "Make sure the backend is running (npm run engine)."
}
