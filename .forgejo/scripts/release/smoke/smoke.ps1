$ErrorActionPreference = 'Stop'

$root = Split-Path -Parent (Split-Path -Parent (Split-Path -Parent (Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path))))
$version = if ($args.Length -gt 0) { $args[0] } else { throw 'missing release version' }
$channel = if ($args.Length -gt 1) { $args[1] } else { 'stable' }
$publicUrl = if ($env:ECTROPY_RELEASES_PUBLIC_URL) { $env:ECTROPY_RELEASES_PUBLIC_URL.TrimEnd('/') } else { 'https://releases.ectropy.perish.uk' }
$tmpdir = Join-Path ([System.IO.Path]::GetTempPath()) ("ectropy-smoke-" + [System.Guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $tmpdir | Out-Null

function Assert-Binary {
    param([string]$Bin, [string]$Clean, [string]$Fault, [string]$Invalid)
    $reported = (& $Bin --version | Out-String).Trim()
    if ($reported -notmatch [regex]::Escape($version.TrimStart('v'))) {
        throw "smoke: version mismatch: $reported"
    }
    $out = (& $Bin $Clean | Out-String).Trim()
    if ($out -ne 'clean') {
        throw "smoke: expected clean, got: $out"
    }
    & $Bin $Fault *> $null
    if ($LASTEXITCODE -eq 0) {
        throw 'smoke: comment fault exited zero'
    }
    & $Bin $Invalid *> $null
    if ($LASTEXITCODE -ne 2) {
        throw "smoke: malformed config exited $LASTEXITCODE instead of 2"
    }
}

function Smoke-Skill {
    param([string]$Bin, [string]$Label)
    $env:ECTROPY_HOME = Join-Path $tmpdir "$Label/data"
    $env:ECTROPY_RELEASES = $publicUrl
    $skill = Join-Path $tmpdir "$Label/agent/skills/ectropy"
    New-Item -ItemType Directory -Force -Path (Split-Path -Parent $skill) | Out-Null
    & $Bin skill install --channel $channel --version $version --path $skill
    if (-not (Test-Path (Join-Path $skill 'SKILL.md'))) {
        throw 'smoke: skill install left no brief'
    }
    if (-not (Test-Path (Join-Path $skill 'metadata.json'))) {
        throw 'smoke: skill install left no metadata'
    }
    & $Bin skill status --channel $channel --version $version *> $null
    & $Bin skill list | Select-String -SimpleMatch $version | Out-Null
    & $Bin skill uninstall
    if (Test-Path $skill) {
        throw "smoke: skill uninstall left $skill"
    }
}

function Smoke-Manager {
    param([string]$Manager, [string]$Label)
    $env:ECTROPY_INSTALL_ROOT = Join-Path $tmpdir "$Label/install"
    $env:ECTROPY_LOCAL_BIN_DIR = Join-Path $tmpdir "$Label/bin"
    New-Item -ItemType Directory -Force -Path $env:ECTROPY_INSTALL_ROOT, $env:ECTROPY_LOCAL_BIN_DIR | Out-Null
    & $Manager install --public-url $publicUrl --channel $channel --version $version --retain=false
    $bin = Join-Path $env:ECTROPY_LOCAL_BIN_DIR 'ectropy.exe'
    Assert-Binary $bin $clean $fault $invalid
    Smoke-Skill $bin $Label
    & $Manager uninstall --version $version
    if (Test-Path (Join-Path $env:ECTROPY_INSTALL_ROOT $version)) {
        throw "smoke: version uninstall left $(Join-Path $env:ECTROPY_INSTALL_ROOT $version)"
    }
    if (Test-Path $bin) {
        throw "smoke: uninstall left $bin"
    }
}

try {
    $clean = Join-Path $tmpdir 'clean'
    $fault = Join-Path $tmpdir 'fault'
    $invalid = Join-Path $tmpdir 'invalid'
    New-Item -ItemType Directory -Path $clean, $fault, $invalid | Out-Null
    Set-Content -Path (Join-Path $clean 'sample.rs') -Value 'fn main() {}'
    Set-Content -Path (Join-Path $fault 'sample.rs') -Value "fn main() {}`n// denied"
    Set-Content -Path (Join-Path $invalid 'ectropy.toml') -Value '[limit'

    $snapshot = Join-Path $tmpdir 'manage.ps1'
    Invoke-WebRequest -Uri "$publicUrl/$channel/versions/$version/manage.ps1" -OutFile $snapshot
    Smoke-Manager $snapshot 'snapshot'
    Smoke-Manager (Join-Path $root 'manage.ps1') 'current'
}
finally {
    Remove-Item -Recurse -Force -ErrorAction SilentlyContinue $tmpdir
}
