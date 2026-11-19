# Starts the app against a throw-away data folder, with the debug port open. NEVER touches your real data or hosts file.
#   scripts\dev\run-sandbox.ps1 [-Exe path] [-Port 9333] [-Lang vi|en]
# Afterwards drive it with: node scripts/dev/cdp.js 9333 http://tauri.localhost/ "<js>"
# Stop it with:  Stop-Process -Id (Get-Content $env:TEMP\disturbless-sandbox.pid)
param(
  [string]$Exe = (Join-Path $PSScriptRoot '..\..\release\Disturbless.exe'),
  [int]$Port = 9333,
  [string]$Lang = 'vi'
)
$data = Join-Path $env:TEMP 'disturbless-sandbox'
Remove-Item $data -Recurse -Force -ErrorAction SilentlyContinue
New-Item -ItemType Directory $data | Out-Null
"{`"settings`":{`"introSeen`":true,`"lang`":`"$Lang`"}}" | Set-Content (Join-Path $data 'disturbless.json') -Encoding utf8
$env:DL_DATA = $data            # separate data folder
$env:DL_NOHOSTS = '1'           # do not edit the hosts file (no UAC prompt)
$env:DL_LOG = Join-Path $data 'trace.log'
$env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = "--remote-debugging-port=$Port"
$p = Start-Process -FilePath (Resolve-Path $Exe) -PassThru
$p.Id | Set-Content (Join-Path $env:TEMP 'disturbless-sandbox.pid')
Write-Host "Disturbless sandbox pid $($p.Id), data $data, debug port $Port"
