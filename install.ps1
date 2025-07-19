# Clean Language Manager Installer for Windows
# This script downloads and installs the latest version of cleanmanager

param(
    [string]$InstallDir = "$env:LOCALAPPDATA\cleanmanager\bin",
    [string]$Repo = "Ivan-Pasco/clean-language-manager"  # Update this to actual repo
)

# Configuration
$BinaryName = "cleanmanager.exe"
$ArchiveName = "cleanmanager-windows-x86_64.zip"

Write-Host "Clean Language Manager Installer" -ForegroundColor Blue
Write-Host "=================================" -ForegroundColor Blue
Write-Host ""

# Function to get latest release version
function Get-LatestVersion {
    try {
        $response = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases/latest"
        return $response.tag_name
    }
    catch {
        Write-Error "Failed to fetch latest version: $_"
        exit 1
    }
}

# Function to download and extract
function Install-CleanManager {
    $version = Get-LatestVersion
    $downloadUrl = "https://github.com/$Repo/releases/download/$version/$ArchiveName"
    
    Write-Host "Platform: Windows x86_64" -ForegroundColor Green
    Write-Host "Version:  $version" -ForegroundColor Green
    Write-Host "Install:  $InstallDir" -ForegroundColor Green
    Write-Host ""
    
    # Create temporary directory
    $tempDir = New-TemporaryFile | ForEach-Object { Remove-Item $_; New-Item -ItemType Directory -Path $_ }
    
    try {
        Write-Host "Downloading cleanmanager..." -ForegroundColor Yellow
        Write-Host "URL: $downloadUrl"
        
        $archivePath = Join-Path $tempDir.FullName $ArchiveName
        Invoke-WebRequest -Uri $downloadUrl -OutFile $archivePath
        
        Write-Host "Extracting archive..." -ForegroundColor Yellow
        Expand-Archive -Path $archivePath -DestinationPath $tempDir.FullName -Force
        
        # Create install directory
        if (!(Test-Path $InstallDir)) {
            New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
        }
        
        # Move binary to install directory
        $binaryPath = Join-Path $tempDir.FullName $BinaryName
        if (!(Test-Path $binaryPath)) {
            Write-Error "Binary not found in archive"
            exit 1
        }
        
        Write-Host "Installing to $InstallDir..." -ForegroundColor Yellow
        Copy-Item $binaryPath -Destination $InstallDir -Force
        
        Write-Host "✅ Clean Language Manager installed successfully!" -ForegroundColor Green
        Write-Host ""
        
        # Check if install directory is in PATH
        $currentPath = [Environment]::GetEnvironmentVariable("PATH", "User")
        if ($currentPath -notlike "*$InstallDir*") {
            Write-Host "⚠️  Installation directory is not in your PATH" -ForegroundColor Yellow
            Write-Host ""
            Write-Host "To add cleanmanager to your PATH:"
            Write-Host "1. Open PowerShell as Administrator"
            Write-Host "2. Run: [Environment]::SetEnvironmentVariable('PATH', `$env:PATH + ';$InstallDir', 'User')" -ForegroundColor Blue
            Write-Host "3. Restart your terminal"
            Write-Host ""
            Write-Host "Alternatively, run cleanmanager directly:"
            Write-Host "  $InstallDir\cleanmanager.exe --help" -ForegroundColor Blue
        }
        else {
            Write-Host "✅ Installation directory is already in your PATH" -ForegroundColor Green
            Write-Host ""
            Write-Host "You can now run:"
            Write-Host "  cleanmanager --help" -ForegroundColor Blue
        }
        
        Write-Host ""
        Write-Host "Next steps:" -ForegroundColor Blue
        Write-Host "1. Run: cleanmanager init"
        Write-Host "2. Run: cleanmanager doctor"
        Write-Host "3. Install a Clean Language version: cleanmanager install <version>"
        Write-Host ""
        Write-Host "For more information: https://github.com/$Repo"
    }
    finally {
        # Cleanup
        Remove-Item $tempDir -Recurse -Force -ErrorAction SilentlyContinue
    }
}

# Main execution
try {
    Install-CleanManager
}
catch {
    Write-Error "Installation failed: $_"
    exit 1
}