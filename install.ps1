$ErrorActionPreference = 'Stop'

$Repo = "Zackriya-Solutions/syncfu"
$Artifact = "syncfu-windows-x86_64.exe"
$BinaryName = "syncfu.exe"

# --- Parse arguments ---
param(
    [string]$Version = ""
)

# --- Resolve version ---
if (-not $Version) {
    Write-Host "info  " -ForegroundColor Green -NoNewline
    Write-Host "Resolving latest version..."
    try {
        $Response = Invoke-WebRequest -Uri "https://github.com/$Repo/releases/latest" `
            -MaximumRedirection 0 -ErrorAction SilentlyContinue
    } catch {
        $Response = $_.Exception.Response
    }
    $RedirectUrl = $Response.Headers.Location
    if (-not $RedirectUrl) {
        $Release = Invoke-RestMethod "https://api.github.com/repos/$Repo/releases/latest"
        $Version = $Release.tag_name -replace '^v', ''
    } else {
        $Version = ($RedirectUrl -split '/v')[-1]
    }
    if (-not $Version) {
        Write-Host "error " -ForegroundColor Red -NoNewline
        Write-Host "Could not determine latest version."
        exit 1
    }
}

# --- Install directory ---
if ($env:SYNCFU_INSTALL_DIR) {
    $InstallDir = $env:SYNCFU_INSTALL_DIR
} else {
    $InstallDir = Join-Path $HOME ".syncfu\bin"
}

$Url = "https://github.com/$Repo/releases/download/v$Version/$Artifact"
$ChecksumUrl = "https://github.com/$Repo/releases/download/v$Version/checksums.txt"

Write-Host ""
Write-Host "  syncfu installer" -ForegroundColor White
Write-Host ""
Write-Host "  Version:  " -ForegroundColor Cyan -NoNewline
Write-Host "v$Version"
Write-Host "  Platform: " -ForegroundColor Cyan -NoNewline
Write-Host "windows/x86_64"
Write-Host "  Install:  " -ForegroundColor Cyan -NoNewline
Write-Host "$InstallDir"
Write-Host ""

# --- Download ---
Write-Host "info  " -ForegroundColor Green -NoNewline
Write-Host "Downloading syncfu v$Version..."

$TmpDir = Join-Path $env:TEMP "syncfu-install"
New-Item -ItemType Directory -Force -Path $TmpDir | Out-Null
$TmpFile = Join-Path $TmpDir $BinaryName

try {
    Invoke-WebRequest -Uri $Url -OutFile $TmpFile
} catch {
    Write-Host "error " -ForegroundColor Red -NoNewline
    Write-Host "Download failed. Check: https://github.com/$Repo/releases/tag/v$Version"
    exit 1
}

# --- Verify checksum ---
Write-Host "info  " -ForegroundColor Green -NoNewline
Write-Host "Verifying checksum..."

try {
    $ChecksumFile = Join-Path $TmpDir "checksums.txt"
    Invoke-WebRequest -Uri $ChecksumUrl -OutFile $ChecksumFile

    $Expected = (Get-Content $ChecksumFile | Where-Object { $_ -match $Artifact }) -replace '\s+.*', ''
    $Actual = (Get-FileHash -Path $TmpFile -Algorithm SHA256).Hash.ToLower()

    if ($Expected -and ($Expected -ne $Actual)) {
        Write-Host "error " -ForegroundColor Red -NoNewline
        Write-Host "Checksum mismatch! Expected: $Expected, Got: $Actual"
        Remove-Item -Recurse -Force $TmpDir
        exit 1
    }
    Write-Host "info  " -ForegroundColor Green -NoNewline
    Write-Host "Checksum verified"
} catch {
    Write-Host "warn  " -ForegroundColor Yellow -NoNewline
    Write-Host "Could not verify checksum, continuing..."
}

# --- Install ---
New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
Move-Item -Force $TmpFile (Join-Path $InstallDir $BinaryName)

Write-Host "info  " -ForegroundColor Green -NoNewline
Write-Host "Installed to $InstallDir\$BinaryName"

# --- Add to PATH ---
$UserPath = [Environment]::GetEnvironmentVariable('Path', 'User')
if ($UserPath -notlike "*$InstallDir*") {
    [Environment]::SetEnvironmentVariable('Path', "$InstallDir;$UserPath", 'User')
    $env:Path = "$InstallDir;$env:Path"
    Write-Host ""
    Write-Host "info  " -ForegroundColor Green -NoNewline
    Write-Host "Added $InstallDir to your PATH."
    Write-Host "      Restart your terminal for PATH changes to take effect."
}

# --- Cleanup ---
Remove-Item -Recurse -Force $TmpDir -ErrorAction SilentlyContinue

# --- Done ---
Write-Host ""
Write-Host "info  " -ForegroundColor Green -NoNewline
Write-Host "Done! Run 'syncfu --help' to get started."
Write-Host ""
