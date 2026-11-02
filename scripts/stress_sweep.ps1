<#
.SYNOPSIS
    Records one Tracy capture of the profiling build per enemy count and kind.

.DESCRIPTION
    Builds the profiling binary once, then for each combination starts tracy-capture, runs the
    game in the stress arena with STRESS_SECONDS so it quits by itself, and waits for the
    capture to finish. Captures go to runs\stress\<kind>_<count>.tracy. The game appends
    one summary line per run to runs\stress_summary.txt.

    Needs tracy-capture on PATH. Run it from anywhere; it works from the repository root.

.PARAMETER Counts
    Total enemies per run. Split evenly across the four kinds, or all one kind with -Kinds.

.PARAMETER Kinds
    all, or warrior, ice_mage, fire_mage, bat.

.PARAMETER Seconds
    Measured seconds per run, after the game's 5 s warm-up.

.PARAMETER MapSeed
    Fixes the map layout and enemy placement across runs.

.EXAMPLE
    .\scripts\stress_sweep.ps1
    .\scripts\stress_sweep.ps1 -Counts 500 -Kinds warrior,ice_mage,fire_mage,bat
#>
param(
    [int[]]$Counts = @(100, 200, 500, 1000),
    [string[]]$Kinds = @("all"),
    [int]$Seconds = 30,
    [int]$MapSeed = 1,
    [string]$OutputDirectory = "runs\stress"
)

$ErrorActionPreference = "Stop"
Set-Location (Split-Path $PSScriptRoot -Parent)

$allKinds = @("warrior", "ice_mage", "fire_mage", "bat")

if (-not (Get-Command tracy-capture -ErrorAction SilentlyContinue)) {
    throw "tracy-capture is not on PATH"
}

foreach ($kind in $Kinds) {
    if ($kind -ne "all" -and $allKinds -notcontains $kind) {
        throw "unknown kind '$kind', expected all, $($allKinds -join ', ')"
    }
}

# Windows PowerShell 5.1 turns cargo's progress output on stderr into errors under "Stop".
$ErrorActionPreference = "Continue"
cargo build --profile profiling --no-default-features --features profile 2>&1 | ForEach-Object { "$_" }
$buildExitCode = $LASTEXITCODE
$ErrorActionPreference = "Stop"
if ($buildExitCode -ne 0) {
    throw "cargo build failed"
}
$gameExecutable = Resolve-Path "target\profiling\baba_yaga.exe"
New-Item -ItemType Directory -Force $OutputDirectory | Out-Null

function Format-ArenaCounts([string]$kind, [int]$count) {
    if ($kind -eq "all") {
        $perKind = [math]::Floor($count / $allKinds.Count)
        return ($allKinds | ForEach-Object { "$_=$perKind" }) -join ","
    }
    return "$kind=$count"
}

$summaryPath = "runs\stress_summary.txt"
foreach ($kind in $Kinds) {
    foreach ($count in $Counts) {
        $captureFile = Join-Path $OutputDirectory "${kind}_${count}.tracy"
        Write-Host "== $kind, $count enemies -> $captureFile"

        # The binary run outside cargo looks for assets next to itself, not in the project.
        $env:BEVY_ASSET_ROOT = (Get-Location).Path
        $env:MAP_SEED = "$MapSeed"
        $env:STRESS_ARENA = Format-ArenaCounts $kind $count
        $env:STRESS_SECONDS = "$Seconds"
        $capture = $null
        $game = $null
        try {
            $capture = Start-Process tracy-capture -ArgumentList @("-o", $captureFile, "-f") `
                -PassThru -NoNewWindow
            Start-Sleep -Seconds 2
            $game = Start-Process $gameExecutable -PassThru
            # Warm-up, measured time, and startup and shutdown slack.
            $gameTimeoutMilliseconds = ($Seconds + 120) * 1000
            if (-not $game.WaitForExit($gameTimeoutMilliseconds)) {
                Write-Warning "game did not exit in time, stopping it"
                Stop-Process $game -Force
            }
            if (-not $capture.WaitForExit(60000)) {
                Write-Warning "tracy-capture did not finish, stopping it"
                Stop-Process $capture -Force
            }
        }
        finally {
            Remove-Item Env:BEVY_ASSET_ROOT, Env:MAP_SEED, Env:STRESS_ARENA, Env:STRESS_SECONDS `
                -ErrorAction SilentlyContinue
        }
    }
}

Write-Host "`nDone. Summary lines:"
if (Test-Path $summaryPath) {
    Get-Content $summaryPath -Tail ($Kinds.Count * $Counts.Count)
}
else {
    Write-Warning "no summary written; a game run probably failed to start"
}
