$ErrorActionPreference = 'Stop'

$command = if ($args.Length -gt 0) { $args[0] } else { 'install' }
[string[]]$remaining = if ($args.Length -gt 1) { $args[1..($args.Length - 1)] } else { @() }

$channel = if ($env:ECTROPY_CHANNEL) { $env:ECTROPY_CHANNEL } else { 'stable' }
$version = if ($env:ECTROPY_VERSION) { $env:ECTROPY_VERSION } else { '' }
$publicUrl = if ($env:ECTROPY_RELEASES_PUBLIC_URL) { $env:ECTROPY_RELEASES_PUBLIC_URL } else { 'https://releases.ectropy.perish.uk' }
$defaultInstallBase = if ($env:LOCALAPPDATA) { $env:LOCALAPPDATA } elseif ($env:HOME) { Join-Path $env:HOME '.local/share' } else { '.' }
$defaultBinBase = if ($env:USERPROFILE) { $env:USERPROFILE } elseif ($env:HOME) { $env:HOME } else { '.' }
$defaultBinLeaf = if ($env:USERPROFILE) { '.local\bin' } else { '.local/bin' }
$installRoot = if ($env:ECTROPY_INSTALL_ROOT) { $env:ECTROPY_INSTALL_ROOT } else { Join-Path $defaultInstallBase 'ectropy' }
$localBinDir = if ($env:ECTROPY_LOCAL_BIN_DIR) { $env:ECTROPY_LOCAL_BIN_DIR } else { Join-Path $defaultBinBase $defaultBinLeaf }
$retain = if ($env:ECTROPY_RETAIN) { $env:ECTROPY_RETAIN } else { '' }

for ($i = 0; $i -lt $remaining.Length; $i++) {
    $arg = $remaining[$i]
    switch -Regex ($arg) {
        '^--channel$' { $i++; $channel = $remaining[$i]; continue }
        '^--channel=(.+)$' { $channel = $Matches[1]; continue }
        '^--version$' { $i++; $version = $remaining[$i]; continue }
        '^--version=(.+)$' { $version = $Matches[1]; continue }
        '^--public-url$' { $i++; $publicUrl = $remaining[$i]; continue }
        '^--public-url=(.+)$' { $publicUrl = $Matches[1]; continue }
        '^--install-root$' { $i++; $installRoot = $remaining[$i]; continue }
        '^--install-root=(.+)$' { $installRoot = $Matches[1]; continue }
        '^--bin-dir$' { $i++; $localBinDir = $remaining[$i]; continue }
        '^--bin-dir=(.+)$' { $localBinDir = $Matches[1]; continue }
        '^--retain$' { $retain = 'true'; continue }
        '^--retain=(.+)$' { $retain = $Matches[1]; continue }
        '^(-h|--help|help)$' {
            @'
ectropy manager

Usage:
  manage.ps1 install [--channel stable|beta] [--version vX.Y.Z] [--retain[=true|false]]
  manage.ps1 uninstall [--version vX.Y.Z]

install leaves exactly one version on disk. Earlier versions are removed once
the new binary is in place and answers --version. Rolling back is
install --version <older>, which fetches that version again; released artifacts
are immutable and always retrievable. Pass --retain to keep what is there.

Options:
  --public-url <url>     release metadata and artifact base URL
  --install-root <path>  versioned install root
  --bin-dir <path>       directory for the ectropy executable

Environment:
  ECTROPY_RELEASES_PUBLIC_URL  # default: https://releases.ectropy.perish.uk
  ECTROPY_CHANNEL
  ECTROPY_VERSION
  ECTROPY_INSTALL_ROOT
  ECTROPY_LOCAL_BIN_DIR
  ECTROPY_RETAIN
'@ | Write-Output
            exit 0
        }
        default { throw "unknown argument: $arg" }
    }
}

function Normalize-Version {
    param([string]$Value)
    return "v$($Value.TrimStart('v'))"
}

function Normalize-Bool {
    param([string]$Value)
    switch -Regex ($Value) {
        '^(true|1|yes|y|on)$' { return $true }
        '^(false|0|no|n|off)$' { return $false }
        default { throw "invalid --retain value: $Value" }
    }
}

function Installed-Versions {
    param([string]$Current)
    if (![System.IO.Directory]::Exists($installRoot)) {
        return @()
    }
    return @(Get-ChildItem -LiteralPath $installRoot -Directory | Where-Object { $_.Name -ne $Current } | ForEach-Object { $_.Name })
}

function Should-Retain {
    param([string[]]$OldVersions)
    if ($OldVersions.Length -eq 0) {
        return $true
    }
    if (![string]::IsNullOrWhiteSpace($retain)) {
        return Normalize-Bool $retain
    }
    return $false
}

function Install-Ectropy {
    $resolvedPublicUrl = $publicUrl.TrimEnd('/')
    $resolvedVersion = $version
    $resolvedLatest = [string]::IsNullOrWhiteSpace($resolvedVersion)
    if ([string]::IsNullOrWhiteSpace($resolvedVersion)) {
        $metadataUrl = "$resolvedPublicUrl/$channel/latest/metadata.json"
        $metadata = Invoke-RestMethod -Uri $metadataUrl
        $resolvedVersion = $metadata.releaseVersion
        if ([string]::IsNullOrWhiteSpace($resolvedVersion)) {
            throw 'failed to resolve latest ectropy version'
        }
    }
    $resolvedVersion = Normalize-Version $resolvedVersion
    if ($channel -eq 'beta' -and $resolvedLatest) {
        try {
            $stable = Invoke-RestMethod -Uri "$resolvedPublicUrl/stable/latest/metadata.json"
            $stableVersion = Normalize-Version $stable.releaseVersion
            $betaBase = [version](($resolvedVersion.TrimStart('v') -split '-')[0])
            $stableBase = [version](($stableVersion.TrimStart('v') -split '-')[0])
            if ($betaBase -lt $stableBase) {
                throw "refusing beta $resolvedVersion older than stable $stableVersion"
            }
        }
        catch {
            $status = $_.Exception.Response.StatusCode.value__
            if ($status -ne 404) {
                throw
            }
        }
    }
    $oldVersions = Installed-Versions $resolvedVersion
    $retainOld = Should-Retain $oldVersions

    $archive = 'ectropy-x86_64-pc-windows-msvc.zip'
    $tmpdir = Join-Path ([System.IO.Path]::GetTempPath()) ("ectropy-" + [System.Guid]::NewGuid().ToString('N'))
    New-Item -ItemType Directory -Path $tmpdir | Out-Null
    try {
        $versionMetadata = Invoke-RestMethod -Uri "$resolvedPublicUrl/$channel/versions/$resolvedVersion/metadata.json"
        if ((Normalize-Version $versionMetadata.releaseVersion) -ne $resolvedVersion) {
            throw "metadata version mismatch: expected $resolvedVersion got $($versionMetadata.releaseVersion)"
        }
        $expected = $versionMetadata.artifacts.windowsX64.sha256
        if ([string]::IsNullOrWhiteSpace($expected)) {
            throw 'metadata missing windowsX64 sha256'
        }
        $archivePath = Join-Path $tmpdir $archive
        Invoke-WebRequest -Uri "$resolvedPublicUrl/$channel/versions/$resolvedVersion/$archive" -OutFile $archivePath
        $actual = (Get-FileHash $archivePath -Algorithm SHA256).Hash.ToLowerInvariant()
        if ($actual -ne $expected.ToLowerInvariant()) {
            throw "checksum mismatch for ${archive}: expected $expected got $actual"
        }
        New-Item -ItemType Directory -Force -Path $installRoot, $localBinDir | Out-Null
        $stage = Join-Path $installRoot (".ectropy-stage-" + [System.Guid]::NewGuid().ToString('N'))
        New-Item -ItemType Directory -Path $stage | Out-Null
        Expand-Archive -LiteralPath $archivePath -DestinationPath $stage -Force
        $stagedBin = Join-Path $stage 'ectropy.exe'
        if (![System.IO.File]::Exists($stagedBin)) {
            throw 'archive missing ectropy.exe'
        }
        $stagedVersion = (& $stagedBin --version | Out-String).Trim()
        if ($stagedVersion -notmatch [regex]::Escape($resolvedVersion.TrimStart('v'))) {
            throw "binary version mismatch: $stagedVersion"
        }
        $versionRoot = Join-Path $installRoot $resolvedVersion
        Remove-Item -Recurse -Force -ErrorAction SilentlyContinue $versionRoot
        Move-Item -LiteralPath $stage -Destination $versionRoot
        $binPath = Join-Path $localBinDir 'ectropy.exe'
        $nextBin = Join-Path $localBinDir (".ectropy-" + [System.Guid]::NewGuid().ToString('N') + '.exe')
        Copy-Item -LiteralPath (Join-Path $versionRoot 'ectropy.exe') -Destination $nextBin
        Move-Item -Force -LiteralPath $nextBin -Destination $binPath
        & $binPath --version

        if (!$retainOld) {
            foreach ($oldVersion in $oldVersions) {
                Remove-Item -Recurse -Force -ErrorAction SilentlyContinue (Join-Path $installRoot $oldVersion)
                Write-Output "removed old ectropy $oldVersion from $installRoot"
            }
        }

        Write-Output "installed ectropy to $(Join-Path $localBinDir 'ectropy.exe')"
    }
    finally {
        Remove-Item -Recurse -Force -ErrorAction SilentlyContinue $tmpdir
    }
}

function Remove-EmptyDir {
    param([string]$Path)
    if ([System.IO.Directory]::Exists($Path)) {
        try {
            Remove-Item -Force -ErrorAction Stop $Path
        }
        catch [System.IO.IOException] {}
    }
}

function Installed-Version {
    $binPath = Join-Path $localBinDir 'ectropy.exe'
    if (![System.IO.File]::Exists($binPath)) {
        return ''
    }
    try {
        $output = & $binPath --version
        if ($output -match 'v?([0-9]+\.[0-9]+\.[0-9]+(?:[-.][A-Za-z0-9]+)*)') {
            return "v$($Matches[1].TrimStart('v'))"
        }
    }
    catch {}
    return ''
}

function Uninstall-Ectropy {
    $binPath = Join-Path $localBinDir 'ectropy.exe'
    if (![string]::IsNullOrWhiteSpace($version)) {
        $normalizedVersion = Normalize-Version $version
        if ((Installed-Version) -eq $normalizedVersion) {
            Remove-Item -Force -ErrorAction SilentlyContinue $binPath
            Write-Output "removed $binPath"
        }
        Remove-Item -Recurse -Force -ErrorAction SilentlyContinue (Join-Path $installRoot $normalizedVersion)
        Remove-EmptyDir $installRoot
        Write-Output "removed ectropy $normalizedVersion from $installRoot"
        return
    }

    Remove-Item -Force -ErrorAction SilentlyContinue $binPath
    Remove-Item -Recurse -Force -ErrorAction SilentlyContinue $installRoot
    Remove-EmptyDir $localBinDir
    Write-Output "removed ectropy from $installRoot and $binPath"
}

switch ($command) {
    { $_ -in @('-h', '--help', 'help') } {
        @'
ectropy manager

Usage:
  manage.ps1 install [--channel stable|beta] [--version vX.Y.Z] [--retain[=true|false]]
  manage.ps1 uninstall [--version vX.Y.Z]

Options:
  --public-url <url>     release metadata and artifact base URL
  --install-root <path>  versioned install root
  --bin-dir <path>       directory for the ectropy executable

Environment:
  ECTROPY_RELEASES_PUBLIC_URL  # default: https://releases.ectropy.perish.uk
  ECTROPY_CHANNEL
  ECTROPY_VERSION
  ECTROPY_INSTALL_ROOT
  ECTROPY_LOCAL_BIN_DIR
  ECTROPY_RETAIN
'@ | Write-Output
    }
    'install' { Install-Ectropy }
    'uninstall' { Uninstall-Ectropy }
    default { throw "unknown command: $command" }
}
