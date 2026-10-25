$ErrorActionPreference = "Stop"

$scriptArguments = @($args)
if ($scriptArguments.Count -gt 1 -or ($scriptArguments.Count -eq 1 -and $scriptArguments[0] -cne "update")) {
    throw "Usage: .\install.ps1 [update]"
}
$mode = if ($scriptArguments.Count -eq 1) { "update" } else { "install" }

$repository = if ($env:WAYPOINT_REPOSITORY) { $env:WAYPOINT_REPOSITORY } else { "hazyio/waypoint" }
$processArchitecture = if ($env:PROCESSOR_ARCHITEW6432) {
    $env:PROCESSOR_ARCHITEW6432
} else {
    $env:PROCESSOR_ARCHITECTURE
}

switch ($processArchitecture) {
    "AMD64" { $target = "x86_64-pc-windows-msvc" }
    "ARM64" { throw "No prebuilt Waypoint release is available for ARM64 Windows. Install Rust and Cargo, then run: cargo install waypoint" }
    default { throw "Unsupported Windows architecture: $processArchitecture. See https://github.com/$repository/releases for supported builds." }
}

$installedBinaryPath = $null
if ($env:INSTALL_DIR) {
    $installDir = [System.IO.Path]::GetFullPath($env:INSTALL_DIR)
    if ($mode -eq "update") {
        $installedBinaryPath = Join-Path $installDir "waypoint.exe"
    }
} elseif ($mode -eq "update") {
    $installedCommand = Get-Command waypoint -CommandType Application -ErrorAction SilentlyContinue
    if (-not $installedCommand) {
        throw "Waypoint is not installed. Run the installer without 'update' to install it first."
    }
    $installedBinaryPath = $installedCommand.Source
    $installDir = Split-Path -Parent $installedBinaryPath
} elseif ($env:LOCALAPPDATA) {
    $installDir = Join-Path $env:LOCALAPPDATA "Programs\Waypoint"
} else {
    $installDir = Join-Path $HOME ".local\bin"
}
$installDir = [System.IO.Path]::GetFullPath($installDir)
$asset = "waypoint-$target.zip"
$downloadUrl = "https://github.com/$repository/releases/latest/download/$asset"
$tempDir = Join-Path ([System.IO.Path]::GetTempPath()) ([guid]::NewGuid().ToString())

try {
    if ($mode -eq "update") {
        if (-not (Test-Path -LiteralPath $installedBinaryPath -PathType Leaf)) {
            throw "Waypoint was not found at $installedBinaryPath."
        }
        Write-Output "Checking for Waypoint updates..."
        $currentVersion = (& $installedBinaryPath --version | Out-String).Trim()
        if ($LASTEXITCODE -ne 0 -or -not $currentVersion) {
            throw "Could not read the installed Waypoint version from $installedBinaryPath."
        }
    }

    New-Item -ItemType Directory -Path $tempDir | Out-Null
    $archivePath = Join-Path $tempDir $asset
    Invoke-WebRequest -Uri $downloadUrl -OutFile $archivePath -UseBasicParsing
    Expand-Archive -LiteralPath $archivePath -DestinationPath $tempDir

    $binaryPath = Join-Path $tempDir "waypoint.exe"
    if (-not (Test-Path -LiteralPath $binaryPath -PathType Leaf)) {
        throw "The release archive must contain a top-level waypoint.exe."
    }

    $releaseVersion = (& $binaryPath --version | Out-String).Trim()
    if ($LASTEXITCODE -ne 0 -or -not $releaseVersion) {
        throw "Could not read the Waypoint version from the downloaded release."
    }

    if ($mode -eq "update" -and $currentVersion -ceq $releaseVersion) {
        Write-Output "Waypoint is up to date ($currentVersion). Exiting."
        return
    }

    if ($mode -eq "update") {
        Write-Output "Updating Waypoint from $currentVersion to $releaseVersion..."
    } else {
        Write-Output "Installing Waypoint $releaseVersion..."
    }

    New-Item -ItemType Directory -Path $installDir -Force | Out-Null
    Copy-Item -LiteralPath $binaryPath -Destination (Join-Path $installDir "waypoint.exe") -Force

    $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
    $userPathEntries = @($userPath -split ";" | Where-Object { $_ })
    if ($userPathEntries -notcontains $installDir) {
        [Environment]::SetEnvironmentVariable("Path", (@($userPathEntries) + $installDir) -join ";", "User")
    }

    if (($env:Path -split ";") -notcontains $installDir) {
        $env:Path = "$installDir;$env:Path"
    }

    if ($mode -eq "update") {
        Write-Output "Updated waypoint to $(Join-Path $installDir 'waypoint.exe')"
    } else {
        Write-Output "Installed waypoint to $(Join-Path $installDir 'waypoint.exe')"
        Write-Output "Run 'waypoint --version' to verify the installation."
    }
} catch {
    throw "Could not install $asset. Ensure a matching asset exists in the latest release. $($_.Exception.Message)"
} finally {
    Remove-Item -LiteralPath $tempDir -Recurse -Force -ErrorAction SilentlyContinue
}
