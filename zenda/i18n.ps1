<#
.SYNOPSIS
  Append Zenda's translation rows (zenda/i18n/<locale>/photocraft.tsv) to a craft-i18n checkout.
.EXAMPLE
  pwsh zenda/i18n.ps1 -CraftI18n ..\craft-i18n
#>
param([Parameter(Mandatory)] [string] $CraftI18n)
$ErrorActionPreference = 'Stop'
$Here = Join-Path $PSScriptRoot 'i18n'
foreach ($dir in Get-ChildItem -Directory $Here) {
  $target = Join-Path $CraftI18n "locales\$($dir.Name)\photocraft.tsv"
  if (-not (Test-Path $target)) { throw "no catalog at $target" }
  $rows = Get-Content -Encoding utf8 (Join-Path $dir.FullName 'photocraft.tsv') | Where-Object { $_ -and -not $_.StartsWith('#') }
  Add-Content -Encoding utf8 -Path $target -Value $rows
  Write-Output "$($dir.Name): $($rows.Count) rows"
}
