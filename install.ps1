#Requires -Version 5.1
param(
    [string]$Version = ""
)

$ErrorActionPreference = 'Stop'

$Repo = "Zackriya-Solutions/syncfu"
$Artifact = "syncfu-windows-x86_64.exe"
$BinaryName = "syncfu.exe"

# --- Resolve version ---
if (-not $Version) {
    Write-Host "info  " -ForegroundColor Green -NoNewline
    Write-Host "Resolving latest version..."
    try {
        $Response = Invoke-WebRequest -Uri "https://github.com/$Repo/releases/latest" `
            -MaximumRedirection 0 -UseBasicParsing -ErrorAction SilentlyContinue
    } catch {
        $Response = $_.Exception.Response
    }
    $RedirectUrl = $Response.Headers.Location
    if (-not $RedirectUrl) {
        $Release = Invoke-RestMethod "https://api.github.com/repos/$Repo/releases/latest"
        $Version = $Release.tag_name -replace '^v', ''
    } elseif ($RedirectUrl -match '/releases/tag/v([0-9]+\.[0-9]+\.[0-9]+)') {
        $Version = $Matches[1]
    } else {
        Write-Host "error " -ForegroundColor Red -NoNewline
        Write-Host "Could not parse version from redirect URL: $RedirectUrl"
        exit 1
    }
    if (-not $Version) {
        Write-Host "error " -ForegroundColor Red -NoNewline
        Write-Host "Could not determine latest version."
        exit 1
    }
}

# --- Validate version format ---
if ($Version -notmatch '^\d+\.\d+\.\d+$') {
    Write-Host "error " -ForegroundColor Red -NoNewline
    Write-Host "Unexpected version format: $Version"
    exit 1
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
$TmpDir = Join-Path $env:TEMP "syncfu-install-$([System.IO.Path]::GetRandomFileName())"
New-Item -ItemType Directory -Force -Path $TmpDir | Out-Null
$TmpFile = Join-Path $TmpDir $BinaryName

try {
    Write-Host "info  " -ForegroundColor Green -NoNewline
    Write-Host "Downloading syncfu v$Version..."

    Invoke-WebRequest -Uri $Url -OutFile $TmpFile -UseBasicParsing

    # --- Verify checksum ---
    Write-Host "info  " -ForegroundColor Green -NoNewline
    Write-Host "Verifying checksum..."

    try {
        $ChecksumFile = Join-Path $TmpDir "checksums.txt"
        Invoke-WebRequest -Uri $ChecksumUrl -OutFile $ChecksumFile -UseBasicParsing

        $EscapedArtifact = [regex]::Escape($Artifact)
        $Lines = @(Get-Content $ChecksumFile | Where-Object { $_ -match "^\S+\s+$EscapedArtifact$" })
        if ($Lines.Count -ne 1) {
            Write-Host "error " -ForegroundColor Red -NoNewline
            Write-Host "Expected exactly 1 checksum entry for $Artifact, found $($Lines.Count)"
            exit 1
        }
        $Expected = ($Lines[0] -replace '\s+.*', '').ToLower()
        $Actual = (Get-FileHash -Path $TmpFile -Algorithm SHA256).Hash.ToLower()

        if ($Expected -ne $Actual) {
            Write-Host "error " -ForegroundColor Red -NoNewline
            Write-Host "Checksum mismatch! Expected: $Expected, Got: $Actual"
            exit 1
        }
        Write-Host "info  " -ForegroundColor Green -NoNewline
        Write-Host "Checksum verified"
    } catch {
        Write-Host "error " -ForegroundColor Red -NoNewline
        Write-Host "Checksum verification failed: $_"
        exit 1
    }

    # --- Install ---
    New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
    Move-Item -Force $TmpFile (Join-Path $InstallDir $BinaryName)

    Write-Host "info  " -ForegroundColor Green -NoNewline
    Write-Host "Installed to $InstallDir\$BinaryName"

    # --- Add to PATH ---
    $UserPath = [Environment]::GetEnvironmentVariable('Path', 'User')
    $PathEntries = $UserPath -split ';'
    if ($InstallDir -notin $PathEntries) {
        [Environment]::SetEnvironmentVariable('Path', "$InstallDir;$UserPath", 'User')
        $env:Path = "$InstallDir;$env:Path"
        Write-Host ""
        Write-Host "info  " -ForegroundColor Green -NoNewline
        Write-Host "Added $InstallDir to your PATH."
        Write-Host "      Restart your terminal for PATH changes to take effect."
    }

    # --- Done ---
    Write-Host ""
    Write-Host "info  " -ForegroundColor Green -NoNewline
    Write-Host "Done! Run 'syncfu --help' to get started."
    Write-Host ""

} catch {
    Write-Host "error " -ForegroundColor Red -NoNewline
    Write-Host "Download failed. Check: https://github.com/$Repo/releases/tag/v$Version"
    exit 1
} finally {
    Remove-Item -Recurse -Force $TmpDir -ErrorAction SilentlyContinue
}
