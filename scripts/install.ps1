#Requires -Version 5.1
$ErrorActionPreference = "Stop"

$Repo = if ($env:TERMVOX_INSTALL_REPO) { $env:TERMVOX_INSTALL_REPO } else { "Jeronimo0228/termvox" }
$Version = if ($env:TERMVOX_VERSION) { $env:TERMVOX_VERSION } else { "latest" }
$Prefix = if ($env:TERMVOX_INSTALL_PREFIX) { $env:TERMVOX_INSTALL_PREFIX } else { "$env:USERPROFILE\.local" }
$InstallSource = if ($env:TERMVOX_INSTALL_SOURCE) { $env:TERMVOX_INSTALL_SOURCE -eq "1" } else { $false }

Write-Host "TermVox installer"
Write-Host "================="

New-Item -ItemType Directory -Force -Path "$Prefix\bin" | Out-Null

function Get-ReleaseTag {
    if ($Version -eq "latest") {
        $release = Invoke-RestMethod "https://api.github.com/repos/$Repo/releases/latest"
        return $release.tag_name
    }
    if ($Version.StartsWith("v")) { return $Version }
    return "v$Version"
}

function Install-FromRelease {
    $tag = Get-ReleaseTag
    $asset = "termvox-$tag-x86_64-pc-windows-msvc.zip"
    $tmp = New-Item -ItemType Directory -Force -Path ([System.IO.Path]::GetTempPath() + [System.Guid]::NewGuid())
    try {
        Write-Host "Downloading $asset ..."
        Invoke-WebRequest -Uri "https://github.com/$Repo/releases/download/$tag/$asset" -OutFile "$tmp\$asset"
        $hashUrl = "https://github.com/$Repo/releases/download/$tag/$asset.sha256"
        try {
            Invoke-WebRequest -Uri $hashUrl -OutFile "$tmp\$asset.sha256"
            $expected = (Get-Content "$tmp\$asset.sha256").Split(" ")[0]
            $actual = (Get-FileHash "$tmp\$asset" -Algorithm SHA256).Hash.ToLowerInvariant()
            if ($expected.ToLowerInvariant() -ne $actual) {
                throw "checksum mismatch for $asset"
            }
        } catch {
            Write-Warning "Skipping checksum verification: $_"
        }
        Expand-Archive -Path "$tmp\$asset" -DestinationPath $tmp -Force
        Copy-Item "$tmp\termvox.exe" "$Prefix\bin\termvox.exe" -Force
    } finally {
        Remove-Item -Recurse -Force $tmp
    }
}

function Install-FromSource {
    if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
        throw "Rust (cargo) is required when no release binary is available. Install from https://rustup.rs"
    }
    $branch = if ($env:TERMVOX_INSTALL_BRANCH) { $env:TERMVOX_INSTALL_BRANCH } else { "main" }
    $tmp = New-Item -ItemType Directory -Force -Path ([System.IO.Path]::GetTempPath() + [System.Guid]::NewGuid())
    try {
        git clone --depth 1 --branch $branch "https://github.com/$Repo.git" "$tmp\termvox"
        cargo install --path "$tmp\termvox\crates\termvox-cli" --force --root $Prefix
    } finally {
        Remove-Item -Recurse -Force $tmp
    }
}

if ($InstallSource) {
    Install-FromSource
} else {
    try {
        Install-FromRelease
    } catch {
        Write-Host "No pre-built release found; building from source..."
        Install-FromSource
    }
}

$env:PATH = "$Prefix\bin;$env:PATH"
termvox models install default
termvox init --preset cursor --force 2>$null
if ($LASTEXITCODE -ne 0) { termvox init --force }

Write-Host ""
Write-Host "Installed TermVox to $Prefix\bin"
Write-Host "Quick start:"
Write-Host "  termvox daemon start --background"
Write-Host "  termvox talk"
