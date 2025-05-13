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

# 4. Remove existing executable if it exists, then copy and rename the new one
if (Test-Path -Path $TargetExePath -PathType Leaf) {
    Write-Host "Removing existing executable at '$TargetExePath'..."
    try {
        Remove-Item -Path $TargetExePath -Force -ErrorAction Stop
        Write-Host "Successfully removed existing executable."
    } catch {
        Write-Warning "Warning: Could not remove existing executable at '$TargetExePath'. Attempting to overwrite. Details: $($_.Exception.Message)"
        # Continue, as Copy-Item -Force might still work
    }
}

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
    $OriginalUserPath = [System.Environment]::GetEnvironmentVariable("Path", "User")
    
    # Create a clean list of existing path entries for the check
    $ExistingPathEntries = $OriginalUserPath -split ';' | ForEach-Object { $_.Trim() } | Where-Object { -not [string]::IsNullOrWhiteSpace($_) }

    if ($ExistingPathEntries -notcontains $InstallDir) {
        $NewUserPath = ""
        if ([string]::IsNullOrWhiteSpace($OriginalUserPath)) {
            # Path was empty, new path is just the install directory
            $NewUserPath = $InstallDir
        } else {
            # Path was not empty, append appropriately
            if ($OriginalUserPath.EndsWith(";")) {
                $NewUserPath = $OriginalUserPath + $InstallDir
            } else {
                $NewUserPath = $OriginalUserPath + ";" + $InstallDir
            }
        }
        
        [System.Environment]::SetEnvironmentVariable("Path", $NewUserPath, "User") # For future sessions
        Write-Host "'$InstallDir' has been added to your user PATH (for new terminal sessions)."
        
        # Attempt to update PATH for the current PowerShell session
        $env:Path = $NewUserPath # For this current session
        Write-Host "Attempting to update PATH for the current PowerShell session..."

        # Verify if the command is found in the current session's updated PATH
        if (Get-Command $AppName -ErrorAction SilentlyContinue) {
            Write-Host "SUCCESS: '$AppName' is now available in THIS PowerShell window."
            Write-Host "For other terminals (CMD, other PowerShell windows), you still need to open a NEW window."
        } else {
            Write-Warning "NOTE: '$AppName' may not be immediately available in this specific PowerShell window, even after attempting to update the PATH."
            Write-Host "IMPORTANT: Please open a NEW CMD or PowerShell window to use '$AppName'. This is usually required for PATH changes to take full effect."
        }
    } else {
        Write-Host "'$InstallDir' is already in your user PATH."
        # Check if usable in current session if already in PATH
        # The $InstallDir might be in the registry's User PATH but not yet in this specific session's $env:Path
        # if this session was started before the PATH was set externally.
        if ($env:Path -like "*$InstallDir*") {
            if (Get-Command $AppName -ErrorAction SilentlyContinue) {
                Write-Host "'$AppName' should be available in this PowerShell session as its directory is in the current session's PATH."
            } else {
                Write-Warning "Although '$InstallDir' is in this session's PATH, '$AppName' was not found. This can sometimes happen. Try a new terminal window."
            }
        } else {
             Write-Warning "'$InstallDir' is in the user PATH registry setting, but not yet reflected in this current session's PATH. A new terminal window is needed to use '$AppName'."
        }
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

# Keep the window open until the user presses Enter
if ($Host.Name -eq 'ConsoleHost') { # Check if running in a console and not ISE/VSCode terminal where this might be annoying
    Read-Host -Prompt "Press Enter to exit"
}
