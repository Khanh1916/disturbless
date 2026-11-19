# Sets up the build environment for this machine WITHOUT touching PATH or the registry (tools live on drive E:).
# Dot-source it:  . .\scripts\dev\env.ps1      Override the location with $env:DL_TOOLS before sourcing.
$tools = if ($env:DL_TOOLS) { $env:DL_TOOLS } else { 'E:\tools' }
$env:RUSTUP_HOME = "$tools\rust\rustup"
$env:CARGO_HOME = "$tools\rust\cargo"
$env:CARGO_TARGET_DIR = "$tools\build-target"
$ra = Get-ChildItem "$tools\rust-anything" -Directory -ErrorAction SilentlyContinue | Select-Object -First 1
$env:PATH = "$tools\rust\cargo\bin;" + $(if ($ra) { "$($ra.FullName);" } else { '' }) + "$env:PATH"
Write-Host "cargo: $(& "$tools\rust\cargo\bin\cargo.exe" --version)  target: $env:CARGO_TARGET_DIR"
