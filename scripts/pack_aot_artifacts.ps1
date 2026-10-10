$ErrorActionPreference = 'Stop'

$repoRoot = Split-Path -Parent $PSScriptRoot
$source = Join-Path $repoRoot 'aot_artifacts'
$archive = Join-Path $repoRoot 'aot_artifacts.zip'

Remove-Item -LiteralPath $archive -Force -ErrorAction SilentlyContinue
Add-Type -AssemblyName System.IO.Compression
Add-Type -AssemblyName System.IO.Compression.FileSystem

$archiveStream = [System.IO.File]::Open($archive, [System.IO.FileMode]::CreateNew)
$zip = $null
try {
    $zip = [System.IO.Compression.ZipArchive]::new(
        $archiveStream,
        [System.IO.Compression.ZipArchiveMode]::Create
    )

    Get-ChildItem -LiteralPath $source -Directory -Recurse -Force | ForEach-Object {
        $entryName = $_.FullName.Substring($source.Length).TrimStart([char[]]@('\', '/'))
        $entryName = $entryName.Replace('\', '/') + '/'
        $zip.CreateEntry($entryName) | Out-Null
    }

    Get-ChildItem -LiteralPath $source -File -Recurse -Force | ForEach-Object {
        $entryName = $_.FullName.Substring($source.Length).TrimStart([char[]]@('\', '/'))
        $entryName = $entryName.Replace('\', '/')
        $entry = $zip.CreateEntry($entryName, [System.IO.Compression.CompressionLevel]::Optimal)
        $inputStream = [System.IO.File]::OpenRead($_.FullName)
        $entryStream = $entry.Open()
        try {
            $inputStream.CopyTo($entryStream)
        }
        finally {
            $entryStream.Dispose()
            $inputStream.Dispose()
        }
    }
}
finally {
    if ($null -ne $zip) {
        $zip.Dispose()
    }
    $archiveStream.Dispose()
}
