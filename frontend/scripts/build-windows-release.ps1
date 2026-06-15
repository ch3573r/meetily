param(
    [ValidateSet("cpu", "vulkan", "cuda", "openblas")]
    [string]$Feature = "vulkan",

    [switch]$CheckOnly,
    [switch]$SkipInstall
)

$ErrorActionPreference = "Stop"

$frontendRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
$tauriRoot = Join-Path $frontendRoot "src-tauri"

Set-Location $frontendRoot

function Assert-Command {
    param(
        [Parameter(Mandatory=$true)]
        [string]$Name
    )

    if (-not (Get-Command $Name -ErrorAction SilentlyContinue)) {
        throw "Missing required command '$Name'. Install it before running the ClawScribe Windows release build."
    }
}

function Assert-File {
    param(
        [Parameter(Mandatory=$true)]
        [string]$Path,

        [Parameter(Mandatory=$true)]
        [string]$Hint
    )

    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
        throw "Missing required file '$Path'. $Hint"
    }
}

$isWindowsHost = [System.Runtime.InteropServices.RuntimeInformation]::IsOSPlatform(
    [System.Runtime.InteropServices.OSPlatform]::Windows
)
if (-not $isWindowsHost) {
    throw "Windows release artifacts must be built on Windows."
}

Assert-Command "node"
Assert-Command "pnpm"
Assert-Command "cargo"

$windowsTarget = "x86_64-pc-windows-msvc"
$llamaHelperBinary = Join-Path $tauriRoot "binaries\llama-helper-$windowsTarget.exe"
Assert-File $llamaHelperBinary "Build it from the repository root with 'cargo build -p llama-helper --release --target $windowsTarget', then copy 'target\$windowsTarget\release\llama-helper.exe' to this path."

$env:NEXT_TELEMETRY_DISABLED = "1"
$env:TAURI_BUNDLE_TARGETS = "msi,nsis"

if (-not $SkipInstall) {
    pnpm install --frozen-lockfile
}

$featureArgs = @()
if ($Feature -ne "cpu") {
    $featureArgs = @("--features", $Feature)
}

if ($CheckOnly) {
    Push-Location $tauriRoot
    try {
        cargo check @featureArgs
    } finally {
        Pop-Location
    }

    pnpm exec tsc --noEmit
    exit 0
}

pnpm build

if ($Feature -eq "cpu") {
    pnpm exec tauri build
} else {
    pnpm exec tauri build -- @featureArgs
}

$bundleRoot = Join-Path $tauriRoot "target\release\bundle"
$artifactPatterns = @(
    Join-Path $bundleRoot "msi\*.msi"
    Join-Path $bundleRoot "nsis\*.exe"
)

Write-Host ""
Write-Host "Windows release artifacts:"
$artifactFiles = @()
foreach ($pattern in $artifactPatterns) {
    Get-ChildItem $pattern -ErrorAction SilentlyContinue | ForEach-Object {
        $artifactFiles += $_
        Write-Host "  $($_.FullName)"
    }
}

if ($artifactFiles.Count -eq 0) {
    throw "No Windows release artifacts were found under '$bundleRoot'."
}

$checksumPath = Join-Path $bundleRoot "SHA256SUMS.txt"
$checksumLines = foreach ($artifact in $artifactFiles | Sort-Object FullName) {
    $hash = Get-FileHash -Algorithm SHA256 -LiteralPath $artifact.FullName
    "$($hash.Hash.ToLowerInvariant())  $($artifact.Name)"
}
$checksumLines | Set-Content -LiteralPath $checksumPath -Encoding ascii

Write-Host ""
Write-Host "SHA-256 checksums:"
Get-Content -LiteralPath $checksumPath | ForEach-Object {
    Write-Host "  $_"
}
Write-Host "  $checksumPath"
