# Commit only if the gate is green, in one command that cannot be half-used.
#
# WHY THIS EXISTS. `gate.ps1` already prints every command's own exit code and
# exits non-zero if any failed — and on 2026-08-12 a commit went in on a RED
# gate anyway, because the commit was CHAINED after the gate in one shell block
# and the printed status was never read. The gate's guarantee survives only if
# something acts on it, and a human reading a number is not something.
#
# That is the same failure as piping a run through `tr` and losing its status:
# the guard was correct and the caller discarded it. gate.ps1 was written
# because describing the procedure was not enough; this exists because gate.ps1
# is not enough on its own either, for exactly the same reason.
#
# USAGE: pwsh -NoProfile -File sessionbench/scripts/green-commit.ps1 -MessageFile msg.txt
[CmdletBinding()]
param(
    # A FILE rather than a string: a here-string handed to pwsh from bash is
    # parsed by the shell first, which killed the first invocation of this.
    [Parameter(Mandatory)][string]$MessageFile,
    [switch]$SkipMsrv
)

$root = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
Set-Location $root

$gate = Join-Path $PSScriptRoot 'gate.ps1'
$global:LASTEXITCODE = $null
if ($SkipMsrv) { & $gate -SkipMsrv } else { & $gate }
# gate.ps1's OWN status, not a pipeline's. Nothing below runs unless it is
# EXPLICITLY zero: an unset $LASTEXITCODE means the gate never ran, which is
# not the same as passing and must not be treated as a failure to interpret.
$code = $LASTEXITCODE
if ($null -eq $code) {
    ''
    'REFUSING TO COMMIT: the gate produced no exit code, so it never ran.'
    exit 1
}
if ($code -ne 0) {
    ''
    "REFUSING TO COMMIT: the gate exited $code."
    exit 1
}

# `git add -A` sweeps the working tree, INCLUDING the message file if it sits
# inside the repo — the first successful run committed its own `.msg.tmp`. Stage
# everything, then unstage the message itself.
git add -A
$msgPath = (Resolve-Path $MessageFile).Path
if ($msgPath.StartsWith($root, [StringComparison]::OrdinalIgnoreCase)) {
    git reset --quiet -- $msgPath
}
git commit -S -F $MessageFile
if ($LASTEXITCODE -ne 0) {
    "commit failed with $LASTEXITCODE"
    exit 1
}
git log --format='%h %G? %s' -1
