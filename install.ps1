#Requires -Version 5.1
param(
    [string]$Version = "",
    [switch]$CliOnly
)

$ErrorActionPreference = 'Stop'

$Repo = "Zackriya-Solutions/syncfu"
$CliArtifact = "syncfu-windows-x86_64.exe"
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
    } elseif ($RedirectUrl -match '/releases/tag/v?([0-9]+\.[0-9]+\.[0-9]+)') {
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

# --- Install directories ---
if ($env:SYNCFU_INSTALL_DIR) {
    $CliDir = $env:SYNCFU_INSTALL_DIR
} else {
    $CliDir = Join-Path $HOME ".syncfu\bin"
}

$BaseUrl = "https://github.com/$Repo/releases/download/v$Version"
# Tauri NSIS installer artifact name
$DesktopArtifact = "syncfu_${Version}_x64-setup.exe"

Write-Host ""
Write-Host "  syncfu installer" -ForegroundColor White
Write-Host ""
Write-Host "  Version:  " -ForegroundColor Cyan -NoNewline
Write-Host "v$Version"
Write-Host "  Platform: " -ForegroundColor Cyan -NoNewline
Write-Host "windows/x86_64"
if (-not $CliOnly) {
    Write-Host "  Desktop:  " -ForegroundColor Cyan -NoNewline
    Write-Host "yes (tray + overlay notifications)"
}
Write-Host "  CLI:      " -ForegroundColor Cyan -NoNewline
Write-Host "$CliDir\$BinaryName"
Write-Host ""

$TmpDir = Join-Path $env:TEMP "syncfu-install-$([System.IO.Path]::GetRandomFileName())"
New-Item -ItemType Directory -Force -Path $TmpDir | Out-Null

try {
    # =============================================
    # 1. Install desktop app (unless -CliOnly)
    # =============================================
    if (-not $CliOnly) {
        Write-Host "info  " -ForegroundColor Green -NoNewline
        Write-Host "Downloading syncfu desktop installer..."

        $InstallerPath = Join-Path $TmpDir $DesktopArtifact
        try {
            Invoke-WebRequest -Uri "$BaseUrl/$DesktopArtifact" -OutFile $InstallerPath -UseBasicParsing
            Write-Host "info  " -ForegroundColor Green -NoNewline
            Write-Host "Running desktop installer (silent)..."

            # NSIS silent install
            Start-Process -FilePath $InstallerPath -ArgumentList "/S" -Wait -NoNewWindow
            Write-Host "info  " -ForegroundColor Green -NoNewline
            Write-Host "Desktop app installed"
        } catch {
            Write-Host "warn  " -ForegroundColor Yellow -NoNewline
            Write-Host "Desktop app download failed. Installing CLI only."
            $CliOnly = $true
        }
    }

    # =============================================
    # 2. Install CLI binary
    # =============================================
    Write-Host "info  " -ForegroundColor Green -NoNewline
    Write-Host "Downloading syncfu CLI..."

    $CliPath = Join-Path $TmpDir $BinaryName
    Invoke-WebRequest -Uri "$BaseUrl/$CliArtifact" -OutFile $CliPath -UseBasicParsing

    # --- Verify checksum ---
    Write-Host "info  " -ForegroundColor Green -NoNewline
    Write-Host "Verifying checksum..."

    $ChecksumFile = Join-Path $TmpDir "checksums.txt"
    Invoke-WebRequest -Uri "$BaseUrl/checksums.txt" -OutFile $ChecksumFile -UseBasicParsing

    $EscapedArtifact = [regex]::Escape($CliArtifact)
    $Lines = @(Get-Content $ChecksumFile | Where-Object { $_ -match "^\S+\s+$EscapedArtifact$" })
    if ($Lines.Count -ne 1) {
        Write-Host "error " -ForegroundColor Red -NoNewline
        Write-Host "Expected exactly 1 checksum entry for $CliArtifact, found $($Lines.Count)"
        exit 1
    }
    $Expected = ($Lines[0] -replace '\s+.*', '').ToLower()
    $Actual = (Get-FileHash -Path $CliPath -Algorithm SHA256).Hash.ToLower()

    if ($Expected -ne $Actual) {
        Write-Host "error " -ForegroundColor Red -NoNewline
        Write-Host "Checksum mismatch! Expected: $Expected, Got: $Actual"
        exit 1
    }
    Write-Host "info  " -ForegroundColor Green -NoNewline
    Write-Host "Checksum verified"

    # --- Install CLI ---
    New-Item -ItemType Directory -Force -Path $CliDir | Out-Null
    Move-Item -Force $CliPath (Join-Path $CliDir $BinaryName)

    Write-Host "info  " -ForegroundColor Green -NoNewline
    Write-Host "CLI installed to $CliDir\$BinaryName"

    # --- Add to PATH ---
    $UserPath = [Environment]::GetEnvironmentVariable('Path', 'User')
    $PathEntries = $UserPath -split ';'
    if ($CliDir -notin $PathEntries) {
        [Environment]::SetEnvironmentVariable('Path', "$CliDir;$UserPath", 'User')
        $env:Path = "$CliDir;$env:Path"
        Write-Host ""
        Write-Host "info  " -ForegroundColor Green -NoNewline
        Write-Host "Added $CliDir to your PATH."
        Write-Host "      Restart your terminal for PATH changes to take effect."
    }

    # =============================================
    # 3. Start desktop app
    # =============================================
    if (-not $CliOnly) {
        $SyncfuExe = Join-Path $env:LOCALAPPDATA "syncfu\syncfu.exe"
        if (Test-Path $SyncfuExe) {
            Write-Host ""
            Write-Host "info  " -ForegroundColor Green -NoNewline
            Write-Host "Starting syncfu (tray + overlay)..."
            Start-Process -FilePath $SyncfuExe -WindowStyle Hidden

            Start-Sleep -Seconds 3
            try {
                $null = Invoke-WebRequest -Uri "http://127.0.0.1:9868/health" -UseBasicParsing -TimeoutSec 2
                Write-Host "info  " -ForegroundColor Green -NoNewline
                Write-Host "syncfu is running - server listening on port 9868"
            } catch {
                Write-Host "warn  " -ForegroundColor Yellow -NoNewline
                Write-Host "syncfu started but server not yet responding. It may take a moment."
            }
        }
    }

    # --- Done ---
    Write-Host ""
    if (-not $CliOnly) {
        Write-Host "  syncfu is installed and running!" -ForegroundColor White
        Write-Host ""
        Write-Host "  Quick test:" -ForegroundColor Cyan
        Write-Host '    syncfu send "Hello from syncfu!"'
    } else {
        Write-Host "  syncfu CLI installed." -ForegroundColor White
        Write-Host ""
        Write-Host "  Quick test:" -ForegroundColor Cyan
        Write-Host '    syncfu send "Hello from syncfu!"'
        Write-Host ""
        Write-Host "  Note: The CLI requires the desktop app running as the server." -ForegroundColor Yellow
        Write-Host "  Re-run this installer without -CliOnly for desktop app."
    }
    Write-Host ""
    Write-Host "info  " -ForegroundColor Green -NoNewline
    Write-Host "Done! Run 'syncfu --help' to get started."
    Write-Host ""

} catch {
    Write-Host "error " -ForegroundColor Red -NoNewline
    Write-Host "Installation failed: $_"
    exit 1
} finally {
    Remove-Item -Recurse -Force $TmpDir -ErrorAction SilentlyContinue
}
