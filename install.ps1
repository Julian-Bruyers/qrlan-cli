#Requires -Version 5.0
<#
.SYNOPSIS
    Installs qrlan for the current user on Windows by downloading the latest release from GitHub.
.DESCRIPTION
    This script fetches the latest release of qrlan from GitHub, downloads the 
    'qrlan-windows-amd64.exe' binary, renames it to 'qrlan.exe', copies it to a 
    user-specific programs directory (%LOCALAPPDATA%\Programs\qrlan),
    and adds this directory to the user's PATH environment variable.
.NOTES
    You may need to adjust your PowerShell execution policy to run this script.
    For example, you can run: Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
    After installation, you must open a new CMD or PowerShell window for the PATH changes to take effect.
#>

param (
    [string]$AppName = "qrlan",
    [string]$GitHubOwner = "julian-bruyers",
    [string]$GitHubRepo = "qrlan-cli",
    [string]$ExpectedAssetSuffix = "windows-amd64.exe" # Used to find the correct asset
)

Write-Host "Starting qrlan installation for the current user..."

# Construct the expected asset name based on AppName and suffix
$ExpectedAssetName = "$($AppName)-$($ExpectedAssetSuffix)" # e.g., qrlan-windows-amd64.exe

# Define temporary download path
$TempDir = $env:TEMP
$DownloadedExePath = Join-Path -Path $TempDir -ChildPath $ExpectedAssetName

# 1. Download the latest release binary from GitHub
Write-Host "Fetching latest release information from GitHub ($GitHubOwner/$GitHubRepo)..."
try {
    $LatestReleaseUrl = "https://api.github.com/repos/$GitHubOwner/$GitHubRepo/releases/latest"
    $ReleaseInfo = Invoke-RestMethod -Uri $LatestReleaseUrl -ErrorAction Stop
    
    $Asset = $ReleaseInfo.assets | Where-Object { $_.name -eq $ExpectedAssetName }

    if (-not $Asset) {
        Write-Error "Error: Could not find asset '$ExpectedAssetName' in the latest release on GitHub."
        Write-Error "Available assets: $($ReleaseInfo.assets | ForEach-Object {$_.name}) -join ', '"
        exit 1
    }

    $DownloadUrl = $Asset.browser_download_url
    Write-Host "Downloading '$ExpectedAssetName' from '$DownloadUrl' to '$DownloadedExePath'..."
    Invoke-WebRequest -Uri $DownloadUrl -OutFile $DownloadedExePath -ErrorAction Stop
    Write-Host "Successfully downloaded '$ExpectedAssetName'."
} catch {
    Write-Error "Error downloading the binary. Details: $($_.Exception.Message)"
    if (Test-Path -Path $DownloadedExePath -PathType Leaf) {
        Remove-Item $DownloadedExePath -Force -ErrorAction SilentlyContinue 
    }
    exit 1
}

# Define installation directory within %LOCALAPPDATA%\Programs
$InstallBaseDir = [System.Environment]::GetFolderPath('LocalApplicationData') # %LOCALAPPDATA%
$InstallDir = Join-Path -Path $InstallBaseDir -ChildPath "Programs\$AppName"
$TargetExePath = Join-Path -Path $InstallDir -ChildPath "$($AppName).exe" # Final name will be qrlan.exe

# 2. Check if downloaded binary exists (redundant if Invoke-WebRequest succeeded, but good practice)
if (-not (Test-Path -Path $DownloadedExePath -PathType Leaf)) {
    Write-Error "Error: Downloaded binary '$DownloadedExePath' not found. This should not happen."
    exit 1
}
Write-Host "Downloaded binary found: $DownloadedExePath"

# 3. Create installation directory if it doesn't exist
if (-not (Test-Path -Path $InstallDir -PathType Container)) {
    Write-Host "Creating installation directory: $InstallDir"
    try {
        New-Item -ItemType Directory -Path $InstallDir -Force -ErrorAction Stop | Out-Null
        Write-Host "Successfully created directory: $InstallDir"
    } catch {
        Write-Error "Error creating directory '$InstallDir'. Please check permissions. Details: $($_.Exception.Message)"
        if (Test-Path -Path $DownloadedExePath -PathType Leaf) { Remove-Item $DownloadedExePath -Force -ErrorAction SilentlyContinue }
        exit 1
    }
} else {
    Write-Host "Installation directory '$InstallDir' already exists."
}

# 4. Copy and rename the executable
Write-Host "Installing '$AppName' to '$TargetExePath'..."
try {
    # Copy the downloaded file (e.g., qrlan-windows-amd64.exe) to the target path (e.g., ...\qrlan.exe)
    Copy-Item -Path $DownloadedExePath -Destination $TargetExePath -Force -ErrorAction Stop
    Write-Host "Successfully copied and renamed to '$TargetExePath'."
} catch {
    Write-Error "Error copying executable. Details: $($_.Exception.Message)"
    if (Test-Path -Path $DownloadedExePath -PathType Leaf) { Remove-Item $DownloadedExePath -Force -ErrorAction SilentlyContinue }
    exit 1
}

# 5. Add installation directory to user's PATH environment variable
Write-Host "Adding '$InstallDir' to user PATH environment variable..."
try {
    $CurrentUserPath = [System.Environment]::GetEnvironmentVariable("Path", "User")
    $PathEntries = $CurrentUserPath -split ';' | ForEach-Object { $_.Trim() } | Where-Object { -not [string]::IsNullOrWhiteSpace($_) }

    if ($PathEntries -notcontains $InstallDir) {
        $NewPath = ($PathEntries + $InstallDir) -join ';'
        [System.Environment]::SetEnvironmentVariable("Path", $NewPath, "User")
        Write-Host "'$InstallDir' has been added to your user PATH."
        Write-Host "IMPORTANT: You must open a new CMD or PowerShell window for the changes to take effect."
        
        # Attempt to update PATH for the current PowerShell session
        $env:Path = $NewPath
        Write-Host "PATH has also been updated for the current PowerShell session."
    } else {
        Write-Host "'$InstallDir' is already in your user PATH."
    }
} catch {
    Write-Warning "Could not automatically update PATH. Details: $($_.Exception.Message)"
    Write-Warning "Please add '$InstallDir' to your PATH environment variable manually to run '$AppName' from anywhere."
}

# 6. Clean up the originally downloaded file from Temp directory
if (Test-Path -Path $DownloadedExePath -PathType Leaf) {
    Write-Host "Cleaning up temporary download: $DownloadedExePath"
    Remove-Item $DownloadedExePath -Force -ErrorAction SilentlyContinue
}

Write-Host ""
Write-Host "$AppName installation finished!"
Write-Host "You should be able to run '$AppName' from any NEW command prompt or PowerShell window."
