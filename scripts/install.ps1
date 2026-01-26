# recurl installer for Windows
# Usage: irm https://recurl.dev/install.ps1 | iex

param(
    [string]$Version = "latest",
    [string]$InstallDir = "$env:LOCALAPPDATA\recurl"
)

$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"

$GitHubRepo = "user/recurl"  # TODO: update with actual repo
$BaseUrl = "https://github.com/$GitHubRepo/releases"

function Write-Info { param($Message) Write-Host "[info] $Message" -ForegroundColor Blue }
function Write-Success { param($Message) Write-Host "[success] $Message" -ForegroundColor Green }
function Write-Warn { param($Message) Write-Host "[warn] $Message" -ForegroundColor Yellow }
function Write-Error { param($Message) Write-Host "[error] $Message" -ForegroundColor Red; exit 1 }

function Get-LatestVersion {
    try {
        $response = Invoke-WebRequest -Uri "$BaseUrl/latest" -MaximumRedirection 0 -ErrorAction SilentlyContinue
    } catch {
        $redirectUrl = $_.Exception.Response.Headers.Location
        if ($redirectUrl) {
            return ($redirectUrl -split '/')[-1]
        }
    }

    # Fallback: parse releases page
    try {
        $releases = Invoke-RestMethod -Uri "https://api.github.com/repos/$GitHubRepo/releases/latest"
        return $releases.tag_name
    } catch {
        return $null
    }
}

function Get-Architecture {
    if ([Environment]::Is64BitOperatingSystem) {
        return "x86_64"
    } else {
        Write-Error "32-bit Windows is not supported"
    }
}

function Add-ToPath {
    param($Path)

    $currentPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if ($currentPath -notlike "*$Path*") {
        [Environment]::SetEnvironmentVariable("Path", "$currentPath;$Path", "User")
        return $true
    }
    return $false
}

function Get-PowerShellProfile {
    if (Test-Path $PROFILE) {
        return $PROFILE
    }
    return $PROFILE.CurrentUserCurrentHost
}

function Main {
    Write-Host ""
    Write-Host "=====================================" -ForegroundColor Green
    Write-Host "       recurl installer (Windows)     " -ForegroundColor Green
    Write-Host "=====================================" -ForegroundColor Green
    Write-Host ""

    # Detect architecture
    $arch = Get-Architecture
    Write-Info "Detected architecture: $arch"

    # Get version
    if ($Version -eq "latest") {
        Write-Info "Fetching latest version..."
        $Version = Get-LatestVersion
        if (-not $Version) {
            Write-Error "Failed to fetch latest version. Set -Version explicitly."
        }
    }
    Write-Info "Installing version: $Version"

    # Install directory
    Write-Info "Install directory: $InstallDir"

    # Create temp directory
    $tempDir = Join-Path $env:TEMP "recurl-install-$(Get-Random)"
    New-Item -ItemType Directory -Path $tempDir -Force | Out-Null

    try {
        # Download
        $archiveName = "recurl-windows-$arch.zip"
        $downloadUrl = "$BaseUrl/download/$Version/$archiveName"
        $archivePath = Join-Path $tempDir $archiveName

        Write-Info "Downloading $archiveName..."
        Invoke-WebRequest -Uri $downloadUrl -OutFile $archivePath

        # Extract
        Write-Info "Extracting..."
        if (Test-Path $InstallDir) {
            Remove-Item -Recurse -Force $InstallDir
        }
        Expand-Archive -Path $archivePath -DestinationPath $InstallDir -Force

        Write-Success "recurl installed to $InstallDir"
        Write-Host ""

        # Verify installation
        Write-Info "Verifying installation..."
        $recurlPath = Join-Path $InstallDir "recurl.exe"
        if (Test-Path $recurlPath) {
            try {
                & $recurlPath --version | Out-Null
                Write-Success "recurl binary works correctly"
            } catch {
                Write-Warn "recurl binary may have issues"
            }
        }
        Write-Host ""

        # Configuration options
        Write-Host "=====================================" -ForegroundColor Yellow
        Write-Host "Configuration options" -ForegroundColor Yellow
        Write-Host "=====================================" -ForegroundColor Yellow
        Write-Host ""
        Write-Host "To use recurl, you have several options:"
        Write-Host ""
        Write-Host "  1. Call recurl directly: $recurlPath https://example.com"
        Write-Host "  2. Add to PATH and use as 'recurl'"
        Write-Host "  3. Create a PowerShell alias for 'curl'"
        Write-Host ""

        # Ask about PATH
        $addPath = Read-Host "Add recurl to PATH? [Y/n]"
        if ($addPath -ne "n" -and $addPath -ne "N") {
            if (Add-ToPath $InstallDir) {
                Write-Success "Added $InstallDir to PATH"
                Write-Info "Restart your terminal for PATH changes to take effect"
            } else {
                Write-Info "Already in PATH"
            }
        }
        Write-Host ""

        # Ask about PowerShell alias
        $profilePath = Get-PowerShellProfile
        $addAlias = Read-Host "Add 'curl' alias to PowerShell profile ($profilePath)? [y/N]"

        if ($addAlias -eq "y" -or $addAlias -eq "Y") {
            # Ensure profile exists
            if (-not (Test-Path $profilePath)) {
                New-Item -ItemType File -Path $profilePath -Force | Out-Null
            }

            $aliasLine = "Set-Alias -Name curl -Value '$recurlPath' -Option AllScope"
            $profileContent = Get-Content $profilePath -Raw -ErrorAction SilentlyContinue

            if ($profileContent -and $profileContent.Contains("Set-Alias -Name curl")) {
                Write-Warn "Alias already exists in profile"
            } else {
                Add-Content -Path $profilePath -Value ""
                Add-Content -Path $profilePath -Value "# recurl - drop-in curl replacement with anti-bot bypass"
                Add-Content -Path $profilePath -Value $aliasLine
                Write-Success "Alias added to $profilePath"
            }

            Write-Host ""
            Write-Info "Run this to apply changes now:"
            Write-Host ""
            Write-Host "    . `$PROFILE"
            Write-Host ""
        }
        Write-Host ""

        # Git Bash instructions
        Write-Host "=====================================" -ForegroundColor Cyan
        Write-Host "Git Bash users" -ForegroundColor Cyan
        Write-Host "=====================================" -ForegroundColor Cyan
        Write-Host ""
        Write-Host "Add this to ~/.bashrc:"
        Write-Host ""
        $escapedPath = $InstallDir.Replace('\', '/')
        Write-Host "    alias curl='$escapedPath/recurl.exe'"
        Write-Host ""

        # Windows limitation note
        Write-Host "=====================================" -ForegroundColor Yellow
        Write-Host "Note: Windows limitation" -ForegroundColor Yellow
        Write-Host "=====================================" -ForegroundColor Yellow
        Write-Host ""
        Write-Host "curl-impersonate is not available on Windows."
        Write-Host "recurl will skip the impersonation layer and go directly"
        Write-Host "to JS preflight when encountering anti-bot protection."
        Write-Host ""

        # Final summary
        Write-Host "=====================================" -ForegroundColor Green
        Write-Host "Installation complete!" -ForegroundColor Green
        Write-Host "=====================================" -ForegroundColor Green
        Write-Host ""
        Write-Host "Installed files:"
        Write-Host "  $InstallDir\recurl.exe   - main binary"
        Write-Host "  $InstallDir\recurld.exe  - daemon"
        Write-Host "  $InstallDir\bin\        - curl engine"
        Write-Host ""
        Write-Host "Documentation: https://github.com/$GitHubRepo#readme"
        Write-Host ""

    } finally {
        # Cleanup
        Remove-Item -Recurse -Force $tempDir -ErrorAction SilentlyContinue
    }
}

# Run main
Main
