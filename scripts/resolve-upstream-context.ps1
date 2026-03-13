$runtimeStatePath = Join-Path $PSScriptRoot "..\cache\client-context.json"

if (Test-Path $runtimeStatePath) {
    try {
        Get-Content $runtimeStatePath -Raw
        exit 0
    } catch {
        # Fall through to best-effort defaults.
    }
}

$clientApp = if ($env:DISTIRA_RUNTIME_CLIENT_APP) { $env:DISTIRA_RUNTIME_CLIENT_APP } else { "VS Code Copilot Chat" }
$upstreamProvider = if ($env:DISTIRA_RUNTIME_UPSTREAM_PROVIDER) { $env:DISTIRA_RUNTIME_UPSTREAM_PROVIDER } else { $null }
$upstreamModel = if ($env:DISTIRA_RUNTIME_UPSTREAM_MODEL) { $env:DISTIRA_RUNTIME_UPSTREAM_MODEL } else { $null }

$payload = @{
    client_app = $clientApp
    upstream_provider = $upstreamProvider
    upstream_model = $upstreamModel
}

$payload | ConvertTo-Json -Compress