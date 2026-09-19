param(
    [Parameter(Mandatory = $true)][string]$Candidate,
    [string]$Cases = "$PSScriptRoot\..\tests\conformance\cases.json",
    [switch]$RequireOracle
)

$ErrorActionPreference = 'Stop'
$oracle = "$env:WINDIR\System32\wbem\wmic.exe"
if (-not (Test-Path $oracle)) {
    $message = "Legacy WMIC oracle not present on $([Environment]::OSVersion.VersionString)"
    if ($RequireOracle) { throw $message }
    Write-Warning $message
    exit 0
}

function Invoke-Program([string]$File, [string[]]$Arguments) {
    $start = [Diagnostics.ProcessStartInfo]::new()
    $start.FileName = $File
    $start.UseShellExecute = $false
    $start.RedirectStandardOutput = $true
    $start.RedirectStandardError = $true
    foreach ($argument in $Arguments) { [void]$start.ArgumentList.Add($argument) }
    $process = [Diagnostics.Process]::Start($start)
    $stdout = $process.StandardOutput.ReadToEnd()
    $stderr = $process.StandardError.ReadToEnd()
    $process.WaitForExit()
    return @{ Stdout = $stdout; Stderr = $stderr; ExitCode = $process.ExitCode }
}

function ConvertTo-SemanticLines([string]$Text) {
    # Encoding and WMIC's historical CRCRLF quirk are tested independently. Live values are
    # compared as trimmed lines because column padding varies with machine data.
    return (($Text -replace "`r`r`n", "`n" -replace "`r`n", "`n") -split "`n" |
        ForEach-Object { $_.TrimEnd() } |
        Where-Object { $_ -ne '' }) -join "`n"
}

$failures = 0
foreach ($case in (Get-Content -Raw $Cases | ConvertFrom-Json)) {
    $expected = Invoke-Program $oracle $case.args
    $actual = Invoke-Program (Resolve-Path $Candidate) $case.args
    $expectedText = ConvertTo-SemanticLines $expected.Stdout
    $actualText = ConvertTo-SemanticLines $actual.Stdout
    if ($expected.ExitCode -ne $actual.ExitCode -or $expectedText -cne $actualText) {
        $failures++
        Write-Error -ErrorAction Continue "Conformance failure: $($case.name)"
        Write-Host "Arguments: $($case.args -join ' ')"
        Write-Host "--- legacy ($($expected.ExitCode))"
        Write-Host $expectedText
        Write-Host "+++ rust ($($actual.ExitCode))"
        Write-Host $actualText
    } else {
        Write-Host "PASS $($case.name)"
    }
}
if ($failures -ne 0) { throw "$failures conformance case(s) failed" }
