param(
    [string]$TauriRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\src-tauri")).Path,
    [string]$Version = "",

    [ValidateSet("cpu", "directml")]
    [string]$Runtime = "cpu"
)

$ErrorActionPreference = "Stop"

function Assert-Command {
    param([Parameter(Mandatory=$true)][string]$Name)

    [void](Resolve-CommandPath -Name $Name)
}

function Resolve-CommandPath {
    param([Parameter(Mandatory=$true)][string]$Name)

    $command = Get-Command $Name -ErrorAction SilentlyContinue
    if ($command) {
        return $command.Source
    }

    if ($Name -eq "cmake") {
        $candidates = @()
        $programFiles = ${env:ProgramFiles}
        $programFilesX86 = ${env:ProgramFiles(x86)}

        if ($programFiles) {
            $candidates += Join-Path $programFiles "CMake\bin\cmake.exe"
        }

        if ($programFilesX86) {
            $vswhere = Join-Path $programFilesX86 "Microsoft Visual Studio\Installer\vswhere.exe"
            if (Test-Path -LiteralPath $vswhere -PathType Leaf) {
                $installationPath = & $vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.CMake.Project -property installationPath
                if ($installationPath) {
                    $candidates += Join-Path $installationPath "Common7\IDE\CommonExtensions\Microsoft\CMake\CMake\bin\cmake.exe"
                }
            }
        }

        if ($programFiles) {
            foreach ($edition in @("Enterprise", "Professional", "Community", "BuildTools")) {
                $candidates += Join-Path $programFiles "Microsoft Visual Studio\2022\$edition\Common7\IDE\CommonExtensions\Microsoft\CMake\CMake\bin\cmake.exe"
            }
        }

        foreach ($candidate in $candidates | Select-Object -Unique) {
            if (Test-Path -LiteralPath $candidate -PathType Leaf) {
                return $candidate
            }
        }
    }

    if (-not $command) {
        throw "Missing required command '$Name'. Install it before staging the sherpa-onnx runtime."
    }
}

function Get-SherpaVersion {
    param([string]$CargoToml)

    $content = Get-Content -LiteralPath $CargoToml -Raw
    $matches = [regex]::Matches($content, 'sherpa-onnx\s*=\s*(?:\{\s*)?(?:version\s*=\s*)?"([^"]+)"')
    if ($matches.Count -eq 0) {
        throw "Could not find sherpa-onnx version in $CargoToml"
    }

    return $matches[$matches.Count - 1].Groups[1].Value
}

function Test-RequiredDlls {
    param(
        [string]$Directory,
        [ValidateSet("cpu", "directml")]
        [string]$Runtime
    )

    $dlls = @(
        "onnxruntime.dll",
        "sherpa-onnx-c-api.dll",
        "sherpa-onnx-cxx-api.dll"
    )

    if ($Runtime -eq "cpu") {
        $dlls += "onnxruntime_providers_shared.dll"
    } elseif ($Runtime -eq "directml") {
        $dlls += "DirectML.dll"
    }

    foreach ($dll in $dlls) {
        if (-not (Test-Path -LiteralPath (Join-Path $Directory $dll) -PathType Leaf)) {
            return $false
        }
    }

    return $true
}

function Invoke-Checked {
    param(
        [Parameter(Mandatory=$true)]
        [string]$FilePath,

        [string[]]$ArgumentList = @()
    )

    & $FilePath @ArgumentList 2>&1 | ForEach-Object {
        Write-Host $_
    }
    if ($LASTEXITCODE -ne 0) {
        throw "'$FilePath $($ArgumentList -join ' ')' failed with exit code $LASTEXITCODE"
    }
}

function Remove-ChildDirectory {
    param(
        [Parameter(Mandatory=$true)]
        [string]$Directory,

        [Parameter(Mandatory=$true)]
        [string]$ParentDirectory
    )

    if (-not (Test-Path -LiteralPath $Directory -PathType Container)) {
        return
    }

    $parentFullPath = [System.IO.Path]::GetFullPath((Resolve-Path -LiteralPath $ParentDirectory).Path)
    $directoryFullPath = [System.IO.Path]::GetFullPath($Directory)
    $comparison = [System.StringComparison]::OrdinalIgnoreCase
    $prefix = $parentFullPath.TrimEnd([System.IO.Path]::DirectorySeparatorChar, [System.IO.Path]::AltDirectorySeparatorChar) + [System.IO.Path]::DirectorySeparatorChar
    if (-not $directoryFullPath.StartsWith($prefix, $comparison)) {
        throw "Refusing to remove '$directoryFullPath' because it is not under '$parentFullPath'."
    }

    Remove-Item -LiteralPath $directoryFullPath -Recurse -Force
}

function Ensure-SherpaSource {
    param(
        [string]$SourceRoot,
        [string]$Version
    )

    $git = Resolve-CommandPath -Name "git"

    $tag = "v$Version"
    if (Test-Path -LiteralPath (Join-Path $SourceRoot ".git") -PathType Container) {
        Invoke-Checked $git -ArgumentList @("-c", "core.longpaths=true", "-C", $SourceRoot, "fetch", "--depth=1", "origin", "refs/tags/${tag}:refs/tags/${tag}")
        Invoke-Checked $git -ArgumentList @("-c", "core.longpaths=true", "-C", $SourceRoot, "checkout", "--force", "tags/$tag")
        return
    }

    New-Item -ItemType Directory -Force -Path (Split-Path -Parent $SourceRoot) | Out-Null
    Invoke-Checked $git -ArgumentList @("-c", "core.longpaths=true", "clone", "--depth=1", "--branch", $tag, "https://github.com/k2-fsa/sherpa-onnx", $SourceRoot)
}

function Build-DirectMlSherpaRuntime {
    param(
        [string]$RepoRoot,
        [string]$Version
    )

    if (-not [System.Runtime.InteropServices.RuntimeInformation]::IsOSPlatform([System.Runtime.InteropServices.OSPlatform]::Windows)) {
        throw "DirectML sherpa-onnx runtime staging must run on Windows."
    }

    $cmake = Resolve-CommandPath -Name "cmake"

    if ($env:CLAWSCRIBE_SHERPA_DIRECTML_CACHE) {
        $cacheBase = Join-Path $env:CLAWSCRIBE_SHERPA_DIRECTML_CACHE "v$Version"
    } elseif ($env:LOCALAPPDATA) {
        $cacheBase = Join-Path $env:LOCALAPPDATA "ClawScribeBuildCache\sherpa-directml\v$Version"
    } else {
        $cacheBase = Join-Path $RepoRoot "target\sdml\v$Version"
    }

    $sourceRoot = Join-Path $cacheBase "s"
    $buildRoot = Join-Path $cacheBase "b"
    $installRoot = Join-Path $cacheBase "i"
    $installLibDir = Join-Path $installRoot "lib"

    if (Test-RequiredDlls -Directory $installLibDir -Runtime "directml") {
        return $installLibDir
    }

    Ensure-SherpaSource -SourceRoot $sourceRoot -Version $Version
    New-Item -ItemType Directory -Force -Path $buildRoot | Out-Null
    New-Item -ItemType Directory -Force -Path $installRoot | Out-Null

    $configureArgs = @(
        "-S", $sourceRoot,
        "-B", $buildRoot,
        "-A", "x64",
        "-D", "CMAKE_BUILD_TYPE=Release",
        "-D", "BUILD_SHARED_LIBS=ON",
        "-D", "CMAKE_INSTALL_PREFIX=$installRoot",
        "-D", "SHERPA_ONNX_ENABLE_DIRECTML=ON",
        "-D", "SHERPA_ONNX_ENABLE_PORTAUDIO=OFF",
        "-D", "SHERPA_ONNX_ENABLE_TTS=OFF",
        "-D", "SHERPA_ONNX_ENABLE_SPEAKER_DIARIZATION=ON",
        "-D", "SHERPA_ONNX_ENABLE_BINARY=OFF",
        "-D", "SHERPA_ONNX_BUILD_C_API_EXAMPLES=OFF",
        "-D", "BUILD_ESPEAK_NG_EXE=OFF"
    )
    Invoke-Checked $cmake -ArgumentList $configureArgs
    Invoke-Checked $cmake -ArgumentList @("--build", $buildRoot, "--config", "Release", "--target", "install", "--", "/m:2")

    if (-not (Test-RequiredDlls -Directory $installLibDir -Runtime "directml")) {
        throw "DirectML sherpa runtime DLLs are incomplete in $installLibDir"
    }

    return $installLibDir
}

$tauriRootPath = (Resolve-Path -LiteralPath $TauriRoot).Path
$repoRoot = (Resolve-Path (Join-Path $tauriRootPath "..\..")).Path
$cargoToml = Join-Path $tauriRootPath "Cargo.toml"

if (-not $Version) {
    $Version = Get-SherpaVersion -CargoToml $cargoToml
}

$cacheRoot = Join-Path $repoRoot "target\sherpa-onnx-prebuilt"
$destDir = Join-Path $tauriRootPath "binaries\sherpa-onnx"
$libDir = $null

if ($Runtime -eq "directml") {
    $libDir = Build-DirectMlSherpaRuntime -RepoRoot $repoRoot -Version $Version
} else {
    $archiveStem = "sherpa-onnx-v$Version-win-x64-shared-MT-Release-lib"
    $libDir = Join-Path $cacheRoot "$archiveStem\lib"

    if (-not (Test-RequiredDlls -Directory $libDir -Runtime "cpu")) {
        New-Item -ItemType Directory -Force -Path $cacheRoot | Out-Null

        $archiveName = "$archiveStem.tar.bz2"
        $archivePath = Join-Path $cacheRoot $archiveName
        $url = "https://github.com/k2-fsa/sherpa-onnx/releases/download/v$Version/$archiveName"

        if (-not (Test-Path -LiteralPath $archivePath -PathType Leaf)) {
            Write-Host "Downloading sherpa-onnx Windows shared runtime from $url"
            Invoke-WebRequest -Uri $url -OutFile $archivePath
        }

        $tar = Get-Command tar -ErrorAction SilentlyContinue
        if (-not $tar) {
            throw "Missing required command 'tar'. Install tar or run a Cargo build once so sherpa-onnx-sys extracts the shared runtime."
        }

        if (Test-Path -LiteralPath (Join-Path $cacheRoot $archiveStem)) {
            Remove-ChildDirectory -Directory (Join-Path $cacheRoot $archiveStem) -ParentDirectory $cacheRoot
        }

        & $tar.Source -xjf $archivePath -C $cacheRoot
        if ($LASTEXITCODE -ne 0) {
            throw "Failed to extract $archivePath"
        }
    }

    if (-not (Test-RequiredDlls -Directory $libDir -Runtime "cpu")) {
        throw "Sherpa runtime DLLs are incomplete in $libDir"
    }
}

New-Item -ItemType Directory -Force -Path $destDir | Out-Null
Get-ChildItem -LiteralPath $destDir -Filter "*.dll" -ErrorAction SilentlyContinue | ForEach-Object {
    Remove-Item -LiteralPath $_.FullName -Force
}
Get-ChildItem -LiteralPath $libDir -Filter "*.dll" | Where-Object {
    $_.Name -ne "DirectML.Debug.dll"
} | ForEach-Object {
    Copy-Item -LiteralPath $_.FullName -Destination (Join-Path $destDir $_.Name) -Force
}

if (-not (Test-RequiredDlls -Directory $destDir -Runtime $Runtime)) {
    throw "Failed to stage complete sherpa runtime DLL set in $destDir"
}

Write-Host "Staged sherpa-onnx $Runtime runtime DLLs:"
Get-ChildItem -LiteralPath $destDir -Filter "*.dll" | ForEach-Object {
    Write-Host "  $($_.Name) ($($_.Length) bytes)"
}
