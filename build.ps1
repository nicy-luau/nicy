param([string]$target = "all")

$TargetMap = @{
    "win-x64"       = "x86_64-pc-windows-gnu"
    "win-arm"       = "aarch64-pc-windows-gnullvm"
    "win-x86"       = "i686-pc-windows-gnu"
    "linux-x64"     = "x86_64-unknown-linux-gnu.2.17"
    "linux-arm"     = "aarch64-unknown-linux-gnu.2.17"
    "linux-x86"     = "i686-unknown-linux-gnu.2.17"
    "mac-x64"       = "x86_64-apple-darwin"
    "mac-arm"       = "aarch64-apple-darwin"
    "android-arm"   = "aarch64-linux-android"
    "android-v7"    = "armv7-linux-androideabi"
}

function Build-Target($name, $rustTarget) {
    $cleanTarget = $rustTarget -replace "\.2\.17",""
    $pureTarget = ($rustTarget -split '\.')[0]
    
    if ($name -like "win-*") {
        $fileName = "nicy.exe"
    } else {
        $fileName = "nicy"
    }

    $binPath = "target/$cleanTarget/release/$fileName"

    if (Test-Path $binPath) {
        Write-Host "`nSkip: $name ja existe" -ForegroundColor Green
        return
    }

    Write-Host "`nCompilando CLI: $name ($rustTarget)" -ForegroundColor Cyan
    rustup target add $pureTarget | Out-Null

    if ($name -like "mac-*") {
        $env:CARGO_PROFILE_RELEASE_STRIP = "false"
    } else {
        $env:CARGO_PROFILE_RELEASE_STRIP = "true"
    }

    if ($name -like "android-*") {
        cross +nightly build --release --target $rustTarget -Z build-std=std,core,alloc
    } else {
        cargo +nightly zigbuild --release --target $rustTarget -Z build-std=std,core,alloc
    }

    if (Test-Path $binPath) {
        Write-Host "Ok: $binPath" -ForegroundColor Green
        
        if ($name -like "mac-*" -or $name -eq "win-arm") {
             Write-Host "UPX Skip: $name" -ForegroundColor Gray
        } else {
             upx --ultra-brute --lzma $binPath
        }
    } else {
        Write-Host "Erro build: $name" -ForegroundColor Red
    }

    $env:CARGO_PROFILE_RELEASE_STRIP = $null
}

if ($target -eq "all") {
    $TargetMap.GetEnumerator() | Sort-Object Name | ForEach-Object { Build-Target $_.Key $_.Value }
} elseif ($TargetMap.ContainsKey($target)) {
    Build-Target $target $TargetMap[$target]
} else {
    Write-Host "Target invalido" -ForegroundColor Red
}