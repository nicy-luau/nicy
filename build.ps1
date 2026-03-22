param(
    [string]$target = "all",
    [ValidateSet("stable", "nightly")]
    [string]$toolchain = "stable",
    [switch]$force
)

$ErrorActionPreference = "Stop"

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

function Build-Target($name, $rustTarget, $toolchain, $manifestPath, $forceBuild) {
    $cleanTarget = $rustTarget -replace "\.2\.17",""
    $pureTarget = ($rustTarget -split '\.')[0]
    $fileName = if ($name -like "win-*") { "nicy.exe" } else { "nicy" }
    $binPath = "target/$cleanTarget/release/$fileName"

    if ((Test-Path $binPath) -and -not $forceBuild) {
        Write-Host "`nSkip: $name ja existe" -ForegroundColor Green
        return
    }

    if ((Test-Path $binPath) -and $forceBuild) {
        Write-Host "`nForce: recompilando $name" -ForegroundColor Yellow
    }

    Write-Host "`nCompilando CLI: $name ($rustTarget)" -ForegroundColor Cyan
    rustup target add $pureTarget | Out-Null

    if ($name -like "mac-*") {
        $env:CARGO_PROFILE_RELEASE_STRIP = "false"
    } else {
        $env:CARGO_PROFILE_RELEASE_STRIP = "true"
    }

    $isWinArmGnu = $rustTarget -eq "aarch64-pc-windows-gnullvm"
    if ($isWinArmGnu) {
        if ([string]::IsNullOrWhiteSpace($env:CFLAGS)) {
            $env:CFLAGS = "-Wno-nullability-completeness"
        } else {
            $env:CFLAGS = "$env:CFLAGS -Wno-nullability-completeness"
        }

        if ([string]::IsNullOrWhiteSpace($env:CXXFLAGS)) {
            $env:CXXFLAGS = "-Wno-nullability-completeness"
        } else {
            $env:CXXFLAGS = "$env:CXXFLAGS -Wno-nullability-completeness"
        }
    }

    $buildExit = 0
    if ($name -like "android-*") {
        if ($toolchain -eq "nightly") {
            cross +nightly build --release --target $rustTarget --manifest-path $manifestPath --target-dir target -Z build-std=std,core,alloc,compiler_builtins,panic_abort
            $buildExit = $LASTEXITCODE
        } else {
            cross build --release --target $rustTarget --manifest-path $manifestPath --target-dir target
            $buildExit = $LASTEXITCODE
        }
    } else {
        if ($toolchain -eq "nightly") {
            cargo +nightly zigbuild --release --target $rustTarget --manifest-path $manifestPath --target-dir target -Z build-std=std,core,alloc,compiler_builtins,panic_abort
            $buildExit = $LASTEXITCODE
        } else {
            cargo zigbuild --release --target $rustTarget --manifest-path $manifestPath --target-dir target
            $buildExit = $LASTEXITCODE
        }
    }

    if ($buildExit -ne 0) {
        Write-Host "Erro build: $name (exit $buildExit)" -ForegroundColor Red
        $env:CARGO_PROFILE_RELEASE_STRIP = $null
        if ($isWinArmGnu) {
            $env:CFLAGS = $null
            $env:CXXFLAGS = $null
        }
        return
    }

    if (Test-Path $binPath) {
        Write-Host "Ok: $binPath" -ForegroundColor Green
        if ($name -like "mac-*" -or $name -eq "win-arm") {
            Write-Host "UPX Skip: $name" -ForegroundColor Gray
        } else {
            upx --ultra-brute --lzma $binPath
            if ($LASTEXITCODE -ne 0) {
                Write-Host "UPX Warn: falhou para $name, mantendo binario sem compressao" -ForegroundColor Yellow
                $global:LASTEXITCODE = 0
            }
        }
    } else {
        Write-Host "Erro build: $name" -ForegroundColor Red
    }

    $env:CARGO_PROFILE_RELEASE_STRIP = $null
    if ($isWinArmGnu) {
        $env:CFLAGS = $null
        $env:CXXFLAGS = $null
    }
}

if ($toolchain -eq "nightly") {
    $manifestPath = "nightly/Cargo.toml"
} else {
    $manifestPath = "Cargo.toml"
}

if ($target -eq "all") {
    $TargetMap.GetEnumerator() | Sort-Object Name | ForEach-Object { Build-Target $_.Key $_.Value $toolchain $manifestPath $force }
} elseif ($TargetMap.ContainsKey($target)) {
    Build-Target $target $TargetMap[$target] $toolchain $manifestPath $force
} else {
    Write-Host "Target invalido" -ForegroundColor Red
}
