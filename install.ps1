# CmdRef - Interactive Command Reference Tool Installer (Windows)
# Usage: irm https://raw.githubusercontent.com/xuanke/command-tool/main/install.ps1 | iex

$ErrorActionPreference = "Stop"

$Repo = "xuanke/command-tool"
$BinaryName = "cmdref.exe"
$Version = if ($args[0]) { $args[0] } else { "latest" }

function Get-InstallDir {
    $dir = Join-Path $env:USERPROFILE ".local\bin"
    if (!(Test-Path $dir)) {
        New-Item -ItemType Directory -Path $dir -Force | Out-Null
    }
    return $dir
}

function Get-Platform {
    $arch = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture
    switch ($arch) {
        "X64"   { return "windows-x86_64" }
        "Arm64" { return "windows-aarch64" }
        default {
            Write-Error "Unsupported architecture: $arch"
            exit 1
        }
    }
}

function Install-CmdRef {
    $platform = Get-Platform
    $installDir = Get-InstallDir

    Write-Host "CmdRef Installer" -ForegroundColor Cyan
    Write-Host "================" -ForegroundColor Cyan
    Write-Host "Platform:   $platform"
    Write-Host "Install to: $installDir"
    Write-Host ""

    # Determine URL
    if ($Version -eq "latest") {
        $url = "https://github.com/$Repo/releases/latest/download/$BinaryName"
    } else {
        $ver = $Version.TrimStart("v")
        $url = "https://github.com/$Repo/releases/download/v$ver/$BinaryName"
    }

    $outFile = Join-Path $installDir $BinaryName

    Write-Host "Downloading cmdref..."
    try {
        Invoke-WebRequest -Uri $url -OutFile $outFile -UseBasicParsing
    } catch {
        Write-Error "Failed to download: $_"
        exit 1
    }

    Write-Host ""
    Write-Host "Installation complete!" -ForegroundColor Green
    Write-Host ""

    # Add to PATH if not present
    $currentPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if ($currentPath -notlike "*$installDir*") {
        $newPath = "$currentPath;$installDir"
        [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
        Write-Host "Added $installDir to your PATH."
        Write-Host "Please restart your terminal for changes to take effect."
    }

    Write-Host ""
    Write-Host "Run 'cmdref' to start!" -ForegroundColor Cyan
}

Install-CmdRef
