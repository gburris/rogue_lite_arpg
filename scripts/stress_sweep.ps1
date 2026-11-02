<#
.SYNOPSIS
    Records Tracy captures of the profiling build in the stress arena, one per enemy count and
    repeat.

.DESCRIPTION
    Builds the profiling binary once, then for each count and repeat starts tracy-capture, runs
    the game in the stress arena with STRESS_SECONDS so it quits by itself, and waits for the
    capture to finish. Captures go to runs\stress\<count>_r<repeat>.tracy, so a rerun overwrites
    them; move the folder between sweeps. The game appends one summary line per run to
    runs\stress_summary.txt, and this script prints the median of the runs' medians per count.

    Needs tracy-capture on PATH. Run it from anywhere; it works from the repository root.

.PARAMETER Counts
    Total enemies per run, split evenly across the enemy kinds.

.PARAMETER Repeats
    Runs per count. Runs that settle into a heavy or calm combat state differ, so use 2-3 and
    read the combat counters in each summary line before comparing.

.PARAMETER Seconds
    Measured seconds per run, after the game's 5 s warm-up.

.PARAMETER CaptureSeconds
    Stops each capture after this many seconds. 0 keeps the whole run. Use 15-20 when you plan
    to export events with tracy-csvexport -u, since a 30 s capture exports about 2.5 GB.

.PARAMETER MapSeed
    Fixes the map layout and enemy placement across runs.

.EXAMPLE
    .\scripts\stress_sweep.ps1
    .\scripts\stress_sweep.ps1 -Counts 500, 1000 -Repeats 3 -CaptureSeconds 20
#>
param(
    [int[]]$Counts = @(100, 200, 500, 1000),
    [int]$Repeats = 1,
    [int]$Seconds = 30,
    [int]$CaptureSeconds = 0,
    [int]$MapSeed = 1,
    [string]$OutputDirectory = "runs\stress"
)

$ErrorActionPreference = "Stop"
Set-Location (Split-Path $PSScriptRoot -Parent)

if (-not (Get-Command tracy-capture -ErrorAction SilentlyContinue)) {
    throw "tracy-capture is not on PATH"
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

$summaryPath = "runs\stress_summary.txt"
$summaryLinesBefore = 0
if (Test-Path $summaryPath) {
    $summaryLinesBefore = @(Get-Content $summaryPath).Count
}

foreach ($count in $Counts) {
    foreach ($repeat in 1..$Repeats) {
        $captureFile = Join-Path $OutputDirectory "${count}_r${repeat}.tracy"
        Write-Host "== $count enemies, repeat $repeat -> $captureFile"

        # The binary run outside cargo looks for assets next to itself, not in the project.
        $env:BEVY_ASSET_ROOT = (Get-Location).Path
        $env:MAP_SEED = "$MapSeed"
        $env:STRESS_ARENA = "$count"
        $env:STRESS_SECONDS = "$Seconds"
        $capture = $null
        $game = $null
        try {
            # Paths with spaces need their own quotes, or no capture is written and nothing
            # reports an error.
            $captureArguments = @("-o", "`"$captureFile`"", "-f")
            if ($CaptureSeconds -gt 0) {
                $captureArguments += @("-s", "$CaptureSeconds")
            }
            $capture = Start-Process tracy-capture -ArgumentList $captureArguments `
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

function Get-Median([double[]]$values) {
    $sorted = $values | Sort-Object
    return $sorted[[math]::Floor(($sorted.Count - 1) / 2)]
}

Write-Host "`nDone."
if (-not (Test-Path $summaryPath)) {
    Write-Warning "no summary written; a game run probably failed to start"
    return
}
$newLines = @(Get-Content $summaryPath | Select-Object -Skip $summaryLinesBefore)
$newLines | ForEach-Object { Write-Host $_ }

# The negative lookbehind skips the fixed_ and no_fixed_ variants of each field.
$runs = foreach ($line in $newLines) {
    if ($line -match 'arena=(\d+) .*?(?<!\w)median_frame_ms=([\d.]+) (?:.*?)(?<!\w)p99_frame_ms=([\d.]+)') {
        [pscustomobject]@{
            Enemies = [int]$Matches[1]
            MedianMs = [double]$Matches[2]
            P99Ms = [double]$Matches[3]
        }
    }
}
if ($runs) {
    Write-Host "`nMedian of the runs' median and p99 frame ms, per enemy count:"
    $runs | Group-Object Enemies | ForEach-Object {
        [pscustomobject]@{
            Enemies = $_.Name
            Runs = $_.Count
            MedianMs = Get-Median ($_.Group.MedianMs)
            P99Ms = Get-Median ($_.Group.P99Ms)
        }
    } | Format-Table -AutoSize
}
