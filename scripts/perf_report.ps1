<#
.SYNOPSIS
    Prints ms per frame (or per call) for a fixed list of zones, one column per Tracy capture.

.DESCRIPTION
    Runs tracy-csvexport on each capture and looks up the zones that matter for this game:
    the main and render threads, the fixed step, physics, sight raycasts, lights, transform
    propagation, and the AI decision and applier systems. Frames are counted from the `main app`
    zone, which includes the warm-up and loading, so compare captures made the same way.

    Per-frame rows are total zone time divided by frames. Per-call rows are the zone's mean time
    per call: the fixed step only runs on some frames, so compare it per call.

    Needs tracy-csvexport on PATH.

.PARAMETER Path
    A folder of .tracy captures, or one or more .tracy files.

.EXAMPLE
    .\scripts\perf_report.ps1 runs\stress
    .\scripts\perf_report.ps1 runs\baseline.tracy, runs\stress\1000_r1.tracy
#>
param(
    [Parameter(Mandatory = $true)]
    [string[]]$Path
)

$ErrorActionPreference = "Stop"

if (-not (Get-Command tracy-csvexport -ErrorAction SilentlyContinue)) {
    throw "tracy-csvexport is not on PATH"
}

# Each zone is a label, a regex over the Tracy zone name (matching zones are summed), and how
# to report it: per frame, or per call.
$zones = @(
    @{ Label = "main app"; Pattern = '^main app$'; Basis = "frame" },
    @{ Label = "render app"; Pattern = '^sub app\{name=RenderApp\}$'; Basis = "frame" },
    @{ Label = "FixedMain"; Pattern = '^schedule\{name=FixedMain\}$'; Basis = "call" },
    @{ Label = "FixedMain (per frame)"; Pattern = '^schedule\{name=FixedMain\}$'; Basis = "frame" },
    @{ Label = "physics"; Pattern = '^schedule\{name=PhysicsSchedule\}$'; Basis = "frame" },
    @{ Label = "substeps"; Pattern = '^schedule\{name=SubstepSchedule\}$'; Basis = "frame" },
    @{ Label = "sight raycasts"; Pattern = '^system\{name="avian2d::spatial_query::raycast"\}$'; Basis = "frame" },
    @{ Label = "light buffers"; Pattern = '^system\{name="bevy_lit::light::render::prepare_light2d_buffers<bevy_lit::light::point_light::PointLight2d>"\}$'; Basis = "frame" },
    @{ Label = "transform propagation"; Pattern = '^system\{name="bevy_transform::systems::parallel::propagate_parent_transforms"\}$'; Basis = "frame" },
    @{ Label = "decide_behavior"; Pattern = '^system\{name="baba_yaga::character::brain::decide::decide_behavior"\}$'; Basis = "frame" },
    @{ Label = "appliers"; Pattern = '^system\{name="baba_yaga::character::brain::(action::equipment_use::apply_equipment_use_requests|motion::[a-z_]+::apply_[a-z_]+)"\}$'; Basis = "frame" }
)

$captures = foreach ($entry in $Path) {
    if (Test-Path $entry -PathType Container) {
        Get-ChildItem $entry -Filter *.tracy | Sort-Object Name
    }
    else {
        Get-Item $entry
    }
}
if (-not $captures) {
    throw "no .tracy captures found in $($Path -join ', ')"
}

# Returns zone name -> @{ TotalNs; Counts }, summed over rows that share a name.
function Read-ZoneStatistics([string]$captureFile) {
    $statistics = @{}
    $lines = & tracy-csvexport -s ";" $captureFile
    if ($LASTEXITCODE -ne 0) {
        throw "tracy-csvexport failed on $captureFile"
    }
    # The name is the first field and the last seven are numbers. The source file in between
    # is never split by `;`, so counting from the end is safe even for odd names.
    foreach ($line in ($lines | Select-Object -Skip 1)) {
        $fields = $line.Split(";")
        if ($fields.Count -lt 10) { continue }
        $name = $fields[0]
        $totalNs = [double]$fields[$fields.Count - 7]
        $counts = [double]$fields[$fields.Count - 5]
        if (-not $statistics.ContainsKey($name)) {
            $statistics[$name] = @{ TotalNs = 0.0; Counts = 0.0 }
        }
        $statistics[$name].TotalNs += $totalNs
        $statistics[$name].Counts += $counts
    }
    return $statistics
}

$columnsByCapture = [ordered]@{}
foreach ($capture in $captures) {
    Write-Host "reading $($capture.Name)"
    $statistics = Read-ZoneStatistics $capture.FullName
    $frames = 0.0
    if ($statistics.ContainsKey("main app")) {
        $frames = $statistics["main app"].Counts
    }
    $column = @{ frames = $frames }
    foreach ($zone in $zones) {
        $totalNs = 0.0
        $counts = 0.0
        foreach ($name in $statistics.Keys) {
            if ($name -match $zone.Pattern) {
                $totalNs += $statistics[$name].TotalNs
                $counts += $statistics[$name].Counts
            }
        }
        $divisor = if ($zone.Basis -eq "frame") { $frames } else { $counts }
        $column[$zone.Label] = if ($divisor -gt 0) { $totalNs / $divisor / 1e6 } else { $null }
    }
    $columnsByCapture[$capture.BaseName] = $column
}

$rows = @([pscustomobject]@{ Zone = "frames"; Unit = "count" })
foreach ($captureName in $columnsByCapture.Keys) {
    $rows[0] | Add-Member -NotePropertyName $captureName `
        -NotePropertyValue ("{0:N0}" -f $columnsByCapture[$captureName].frames)
}
foreach ($zone in $zones) {
    $unit = if ($zone.Basis -eq "frame") { "ms/frame" } else { "ms/call" }
    $row = [pscustomobject]@{ Zone = $zone.Label; Unit = $unit }
    foreach ($captureName in $columnsByCapture.Keys) {
        $value = $columnsByCapture[$captureName][$zone.Label]
        $text = if ($null -eq $value) { "-" } else { "{0:N3}" -f $value }
        $row | Add-Member -NotePropertyName $captureName -NotePropertyValue $text
    }
    $rows += $row
}
$rows | Format-Table -AutoSize
