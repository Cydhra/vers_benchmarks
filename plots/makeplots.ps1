if (-not (Test-Path $PSScriptRoot/plots)) {
    New-Item -ItemType Directory $PSScriptRoot/plots > $null
}

typst compile $PSScriptRoot/rank.typ --root=.. $PSScriptRoot/plots/rank.pdf
typst compile $PSScriptRoot/select.typ --root=.. $PSScriptRoot/plots/select.pdf
typst compile $PSScriptRoot/elias_fano.typ --root=.. $PSScriptRoot/plots/elias_fano.pdf