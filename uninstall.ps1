#Requires -Version 5.1
$ErrorActionPreference = 'Stop'

Write-Host ""
Write-Host "  syncfu uninstaller" -ForegroundColor White
Write-Host ""

$Removed = 0

# --- Stop running processes ---
$Procs = Get-Process -Name "syncfu" -ErrorAction SilentlyContinue
if ($Procs) {
    Write-Host "info  " -ForegroundColor Green -NoNewline
    Write-Host "Stopping syncfu processes..."
    $Procs | Stop-Process -Force
    Start-Sleep -Seconds 1
}

# --- Remove desktop app (NSIS install location) ---
$AppDir = Join-Path $env:LOCALAPPDATA "syncfu"
if (Test-Path $AppDir) {
    $Uninstaller = Join-Path $AppDir "Uninstall syncfu.exe"
    if (Test-Path $Uninstaller) {
        Write-Host "info  " -ForegroundColor Green -NoNewline
        Write-Host "Running desktop app uninstaller..."
        Start-Process -FilePath $Uninstaller -ArgumentList "/S" -Wait -NoNewWindow
    } else {
        Remove-Item -Recurse -Force $AppDir
    }
    Write-Host "info  " -ForegroundColor Green -NoNewline
    Write-Host "Removed desktop app"
    $Removed++
}

# --- Remove CLI binary ---
$CliDir = Join-Path $HOME ".syncfu\bin"
if (Test-Path (Join-Path $CliDir "syncfu.exe")) {
    Remove-Item -Force (Join-Path $CliDir "syncfu.exe")
    Write-Host "info  " -ForegroundColor Green -NoNewline
    Write-Host "Removed $CliDir\syncfu.exe"
    $Removed++
}

# --- Clean up ~/.syncfu directory ---
$SyncfuDir = Join-Path $HOME ".syncfu"
if (Test-Path $SyncfuDir) {
    Remove-Item -Recurse -Force $SyncfuDir
    Write-Host "info  " -ForegroundColor Green -NoNewline
    Write-Host "Removed ~/.syncfu"
}

# --- Remove from PATH ---
$UserPath = [Environment]::GetEnvironmentVariable('Path', 'User')
if ($UserPath -and $UserPath -like '*\.syncfu\bin*') {
    $NewPath = ($UserPath -split ';' | Where-Object { $_ -notlike '*\.syncfu\bin*' }) -join ';'
    [Environment]::SetEnvironmentVariable('Path', $NewPath, 'User')
    Write-Host "info  " -ForegroundColor Green -NoNewline
    Write-Host "Removed syncfu from PATH"
}

# --- Done ---
Write-Host ""
if ($Removed -gt 0) {
    Write-Host "info  " -ForegroundColor Green -NoNewline
    Write-Host "syncfu has been uninstalled."
} else {
    Write-Host "warn  " -ForegroundColor Yellow -NoNewline
    Write-Host "No syncfu installation found."
}
Write-Host ""
