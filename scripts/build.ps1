# OpenKeyring CLI Cross-Platform Build Script (PowerShell)
# Supports: macOS (Intel/ARM), Windows, Linux

param(
    [switch]$Help,
    [switch]$Package,
    [switch]$Version,
    [string[]]$Targets = @()
)

$ErrorActionPreference = "Stop"

# Configuration
$ProjectName = "ok"
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectRoot = Split-Path -Parent $ScriptDir
$VersionOutput = (Select-String -Path "$ProjectRoot\keyring-cli\Cargo.toml" -Pattern '^version' | Select-Object -First 1).Line
if ($VersionOutput -match 'version\s*=\s*"([^"]+)"') {
    $Version = $matches[1]
} else {
    $Version = "0.1.0"
}

$OutputDir = "$ProjectRoot\target\release"

# Print banner
function Print-Banner {
    Write-Host @"
 _               _              ___ ___
| |_____   _____| |__          / __/ _ \
| '_ \ \ / / _ \ '_ \ _____  | (_| (_) |
|_.__/\ V /  __/ |_) |_____|  \___\___/
|_|    \_/ \___|_.__/
"@ -ForegroundColor Cyan
    Write-Host "[INFO] OpenKeyring CLI v$Version - Cross-Platform Build Script" -ForegroundColor Cyan
}

# Logging functions
function Log-Info {
    param([string]$Message)
    Write-Host "[INFO] $Message" -ForegroundColor Cyan
}

function Log-Success {
    param([string]$Message)
    Write-Host "[SUCCESS] $Message" -ForegroundColor Green
}

function Log-Warn {
    param([string]$Message)
    Write-Host "[WARN] $Message" -ForegroundColor Yellow
}

function Log-Error {
    param([string]$Message)
    Write-Host "[ERROR] $Message" -ForegroundColor Red
}

# Detect host platform
function Detect-Host {
    $OS = if ($IsWindows -or $env:OS -like "*Windows*") { "windows" }
          elseif ($IsMacOS) { "macos" }
          elseif ($IsLinux) { "linux" }
          else { "unknown" }

    $Arch = if ([Environment]::Is64BitOperatingSystem) { "x86_64" } else { "unknown" }

    Log-Info "Detected host: ${OS}-${Arch}"

    return @{ OS = $OS; Arch = $Arch }
}

# Install Rust target
function Install-Target {
    param([string]$Target)

    $installedTargets = rustup target list --installed 2>$null
    if ($installedTargets -match $Target) {
        Log-Info "Target already installed: $Target"
    } else {
        Log-Info "Installing Rust target: $Target"
        rustup target add $Target
    }
}

# Build for Windows
function Build-Windows {
    param([string]$Arch = "x86_64")

    Log-Info "Building for Windows (${Arch})..."

    Install-Target "${Arch}-pc-windows-msvc"

    Push-Location "$ProjectRoot\keyring-cli"
    cargo build --target "${Arch}-pc-windows-msvc" --release
    Pop-Location

    $BinDir = "$OutputDir\${Arch}-pc-windows-msvc\release"
    $ExeName = "${ProjectName}.exe"

    Log-Success "Windows (${Arch}) binary: $BinDir\$ExeName"
}

# Build for Linux (cross-compile from Windows)
function Build-Linux {
    param([string]$Arch = "x86_64")

    Log-Info "Building for Linux (${Arch})..."

    Install-Target "${Arch}-unknown-linux-gnu"

    # Check for cross-compilation tools
    $mingwExists = Get-Command "x86_64-linux-gnu-gcc" -ErrorAction SilentlyContinue

    if (-not $mingwExists) {
        Log-Warn "Linux cross-compilation toolchain not found."
        Log-Warn "Install with: 'scoop install mingw-winlibs' or similar"
        Log-Warn "Skipping Linux build from Windows."
        return
    }

    Push-Location "$ProjectRoot\keyring-cli"
    $env:CC = "x86_64-linux-gnu-gcc"
    $env:CXX = "x86_64-linux-gnu-g++"
    cargo build --target "${Arch}-unknown-linux-gnu" --release
    Remove-Item Env:\CC, Env:\CXX -ErrorAction SilentlyContinue
    Pop-Location

    $BinDir = "$OutputDir\${Arch}-unknown-linux-gnu\release"

    Log-Success "Linux (${Arch}) binary: $BinDir\$ProjectName"
}

# Build for macOS (cross-compile - not recommended from Windows)
function Build-macOS {
    Log-Warn "macOS cross-compilation from Windows is not supported."
    Log-Warn "Please run the build script on a Mac."
}

# Package release archives
function Package-Release {
    Log-Info "Creating release archives..."

    $PkgDir = "$ProjectRoot\target\packages"
    Remove-Item -Recurse -Force $PkgDir -ErrorAction SilentlyContinue
    New-Item -ItemType Directory -Path $PkgDir -Force | Out-Null

    # Windows x86_64
    $WinBin = "$OutputDir\x86_64-pc-windows-msvc\release\${ProjectName}.exe"
    if (Test-Path $WinBin) {
        $Archive = "$PkgDir\${ProjectName}-${Version}-windows-x86_64.zip"
        Compress-Archive -Path $WinBin -DestinationPath $Archive -Force
        Log-Success "Created: $Archive"
    }

    # Linux x86_64
    $LinuxBin = "$OutputDir\x86_64-unknown-linux-gnu\release\${ProjectName}"
    if (Test-Path $LinuxBin) {
        $Archive = "$PkgDir\${ProjectName}-${Version}-linux-x86_64.zip"
        Compress-Archive -Path $LinuxBin -DestinationPath $Archive -Force
        Log-Success "Created: $Archive"
    }
}

# Show usage
function Show-Usage {
    Write-Host @"
Usage: .\build.ps1 [OPTIONS] [TARGETS]

OPTIONS:
    -h, -Help           Show this help message
    -p, -Package        Create release archives after building
    -v, -Version        Show version information

TARGETS:
    macos               Build all macOS variants (not supported from Windows)
    linux               Build for Linux x86_64 (requires mingw)
    linux-arm64         Build for Linux ARM64 (requires mingw)
    windows             Build for Windows x86_64 (default)
    windows-arm64       Build for Windows ARM64 (native ARM64 Windows)
    all                 Build all supported platforms (default)

EXAMPLES:
    .\build.ps1                 # Build Windows only
    .\build.ps1 windows linux   # Build Windows and Linux
    .\build.ps1 -p windows      # Build and create archive
    .\build.ps1 -v              # Show version

CROSS-COMPILATION NOTES:
    Windows builds: Native, works with MSVC toolchain
    Linux builds: Requires mingw-winlibs or similar
    macOS builds: Not supported from Windows, use macOS

For cross-compilation tools:
    scoop install mingw-winlibs
    or: choco install mingw-w64

"@
}

# Main build function
function Main {
    if ($Help) {
        Show-Usage
        exit 0
    }

    if ($Version) {
        Write-Host "OpenKeyring CLI v$Version"
        exit 0
    }

    Print-Banner
    $HostInfo = Detect-Host

    # Default targets
    if ($Targets.Count -eq 0) {
        $Targets = @("windows")
    }

    # Process 'all' target
    if ($Targets -contains "all") {
        if ($HostInfo.OS -eq "windows") {
            $Targets = @("windows", "windows-arm64", "linux")
        } else {
            $Targets = @("windows", "windows-arm64", "linux", "macos")
        }
    }

    # Build for each target
    foreach ($Target in $Targets) {
        switch ($Target) {
            "windows" {
                Build-Windows
            }
            "windows-arm64" {
                Build-Windows "aarch64"
            }
            "linux" {
                Build-Linux
            }
            "linux-arm64" {
                Build-Linux "aarch64"
            }
            "macos" {
                Build-macOS
            }
            default {
                Log-Error "Unknown target: $Target"
                Show-Usage
                exit 1
            }
        }
    }

    # Package if requested
    if ($Package) {
        Package-Release
    }

    Log-Success "Build complete!"
    Log-Info "Binaries location: $OutputDir"
}

# Run main
Main
