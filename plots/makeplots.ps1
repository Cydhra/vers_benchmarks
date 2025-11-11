if (-not (Test-Path $PSScriptRoot/plots)) {
    New-Item -ItemType Directory $PSScriptRoot/plots > $null
}

typst compile $PSScriptRoot/rank.typ --root=.. $PSScriptRoot/plots/rank.pdf
typst compile $PSScriptRoot/select.typ --root=.. $PSScriptRoot/plots/select.pdf