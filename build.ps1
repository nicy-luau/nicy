param(
    [string]$target = "all",
    [switch]$force
)

$ErrorActionPreference = "Stop"

$TargetMap = @{
    "android-arm" = "aarch64-linux-android"
    "android-v7"  = "armv7-linux-androideabi"
    "linux-arm"   = "aarch64-unknown-linux-gnu.2.17"
    "linux-x64"   = "x86_64-unknown-linux-gnu.2.17"
    "linux-x86"   = "i686-unknown-linux-gnu.2.17"
    "mac-arm"     = "aarch64-apple-darwin"
    "mac-x64"     = "x86_64-apple-darwin"
    "win-arm"     = "aarch64-pc-windows-gnullvm"
    "win-x64"     = "x86_64-pc-windows-gnu"
    "win-x86"     = "i686-pc-windows-gnu"
}

function Assert-Command([string]$name) {
    if (-not (Get-Command $name -ErrorAction SilentlyContinue)) {
        throw "Comando obrigatorio nao encontrado: $name"
    }
}

function Invoke-Build([string]$name, [string]$rustTarget, [switch]$forceBuild) {
    $cleanTarget = $rustTarget -replace "\.2\.17", ""
    $pureTarget = ($rustTarget -split '\.')[0]
    $fileName = if ($name -like "win-*") { "nicy.exe" } else { "nicy" }
    $binPath = "target/$cleanTarget/release/$fileName"

    if ((Test-Path $binPath) -and -not $forceBuild) {
        Write-Host "`nSkip: $name ja existe" -ForegroundColor Green
        return $true
    }

    if ((Test-Path $binPath) -and $forceBuild) {
        Write-Host "`nForce: recompilando $name" -ForegroundColor Yellow
    }

    Write-Host "`nCompilando CLI: $name ($rustTarget)" -ForegroundColor Cyan
    rustup target add $pureTarget | Out-Null
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Erro ao instalar target Rust: $pureTarget" -ForegroundColor Red
        return $false
    }

    $isWinArmGnu = $rustTarget -eq "aarch64-pc-windows-gnullvm"
    $oldCFlags = $env:CFLAGS
    $oldCxxFlags = $env:CXXFLAGS

    try {
        if ($isWinArmGnu) {
            $env:CFLAGS = if ([string]::IsNullOrWhiteSpace($oldCFlags)) { "-Wno-nullability-completeness" } else { "$oldCFlags -Wno-nullability-completeness" }
            $env:CXXFLAGS = if ([string]::IsNullOrWhiteSpace($oldCxxFlags)) { "-Wno-nullability-completeness" } else { "$oldCxxFlags -Wno-nullability-completeness" }
        }

        if ($name -like "android-*") {
            Assert-Command "cross"
            cross build --release --target $rustTarget --manifest-path Cargo.toml --target-dir target
        } else {
            Assert-Command "cargo"
            cargo zigbuild --release --target $rustTarget --manifest-path Cargo.toml --target-dir target
        }

        if ($LASTEXITCODE -ne 0) {
            Write-Host "Erro build: $name (exit $LASTEXITCODE)" -ForegroundColor Red
            return $false
        }

        if (-not (Test-Path $binPath)) {
            Write-Host "Erro: binario nao encontrado apos build: $binPath" -ForegroundColor Red
            return $false
        }

        Write-Host "Ok: $binPath" -ForegroundColor Green
        return $true
    }
    finally {
        $env:CFLAGS = $oldCFlags
        $env:CXXFLAGS = $oldCxxFlags
    }
}

$targetsToBuild = if ($target -eq "all") {
    $TargetMap.GetEnumerator() | Sort-Object Name
} elseif ($TargetMap.ContainsKey($target)) {
    @([PSCustomObject]@{ Name = $target; Value = $TargetMap[$target] })
} else {
    throw "Target invalido: $target"
}

$failed = New-Object System.Collections.Generic.List[string]
foreach ($entry in $targetsToBuild) {
    $ok = Invoke-Build -name $entry.Name -rustTarget $entry.Value -forceBuild:$force
    if (-not $ok) {
        $failed.Add($entry.Name)
    }
}

if ($failed.Count -gt 0) {
    Write-Host "Falhas: $($failed -join ', ')" -ForegroundColor Red
    exit 1
}

Write-Host "Build finalizado sem falhas" -ForegroundColor Green
