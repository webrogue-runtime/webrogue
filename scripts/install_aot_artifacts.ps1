$ErrorActionPreference = 'Stop'

$repoRoot = Split-Path -Parent $PSScriptRoot
$source = Join-Path $repoRoot 'aot_artifacts'
$destination = Join-Path $env:LOCALAPPDATA 'Webrogue'
$destination = Join-Path $destination 'aot_artifacts'
Remove-Item -Recurse -Force $destination
Copy-Item -Recurse $source $destination
