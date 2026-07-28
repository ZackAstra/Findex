Write-Host "=== Building Findex v0.0.1 ==="
$ErrorActionPreference = "Stop"

# Build the release binary
Write-Host "Building release binary..."
cargo build --release -p findex-cli

if ($LASTEXITCODE -eq 0) {
    Write-Host "Build successful!"
    $target = "D:\Findows\target\release\findex.exe"
    if (Test-Path $target) {
        $size = (Get-Item $target).Length
        Write-Host "Binary: $target ($([math]::Round($size/1KB)) KB)"
    }
} else {
    Write-Host "Build failed!"
    exit 1
}
