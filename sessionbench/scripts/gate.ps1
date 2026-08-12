# The commit gate, run so that a failure cannot read as a pass.
#
# WHY THIS EXISTS AS A FILE. CLAUDE.md describes the procedure — send the whole
# output to a uniquely named log and search the log — and describing it was not
# enough. On 2026-08-12 a commit went in on `test=101` because the inline form
# filtered `FAILED|^error|panicked` and then took the first three matches, which
# were `test result: ok` lines from earlier suites. THE SURVIVORS OF THE FILTER
# ALL SAID OK WHILE THE EXIT CODE SAID FAILURE, printed one line above, unread.
#
# The inline form is cheaper every time until the once it costs a bad commit, so
# the fix is not vigilance but making the correct form the cheap one.
#
# WHAT IT GUARANTEES:
#   - every command's own exit code, not a pipeline's last member;
#   - a UNIQUE log per invocation, because a fixed path let two concurrent runs
#     collide on 2026-08-11 and the second captured nothing while reporting the
#     first's zero;
#   - failure lines printed from the log rather than from a live filter, and
#     only for the commands that actually failed;
#   - a non-zero exit if ANY step failed, so a caller cannot miss it either.
#
# USAGE: pwsh -NoProfile -File sessionbench/scripts/gate.ps1
[CmdletBinding()]
param(
    # Skip the MSRV check, which is the slow one and needs the 1.88.0 toolchain.
    [switch]$SkipMsrv
)

$env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"
Set-Location (Split-Path -Parent (Split-Path -Parent $PSScriptRoot))

$stamp = Get-Date -Format 'yyyyMMdd-HHmmss'
$log = Join-Path $env:TEMP "coggy-gate-$stamp-$PID.log"
"gate log: $log"

$steps = @(
    @{ Name = 'fmt'; Args = @('fmt', '--check') },
    @{ Name = 'clippy'; Args = @('clippy', '--all-targets', '--', '-D', 'warnings') },
    @{ Name = 'test'; Args = @('test') }
)
if (-not $SkipMsrv) {
    # A separate toolchain, so it is a separate argv rather than a flag.
    $steps += @{ Name = 'msrv'; Args = @('+1.88.0', 'check', '--all-targets', '--locked') }
}

$failed = @()
foreach ($s in $steps) {
    "--- $($s.Name) ---" | Out-File -FilePath $log -Append -Encoding utf8
    & cargo @($s.Args) *>> $log
    # `cargo`'s own status. Not a pipeline's last native member, which is what
    # made a PowerShell script report 0 after exiting 1 on 2026-08-11.
    $code = $LASTEXITCODE
    "{0,-8} exit={1}" -f $s.Name, $code
    if ($code -ne 0) { $failed += $s.Name }
}

if ($failed.Count -eq 0) {
    'GATE GREEN'
    exit 0
}

''
"GATE RED: $($failed -join ', ')"
''
# Read the log, not a live stream. Everything is already on disk, so nothing
# here depends on a pattern written before the failure was known.
Select-String -Path $log -Pattern 'error(\[|:)|test result: FAILED|panicked at|^failures:' |
    Select-Object -First 25 |
    ForEach-Object { '  ' + $_.Line.Trim() }
''
"full output: $log"
exit 1
