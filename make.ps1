$version = "0.0.0"
Function build_and_package {
    Foreach ($target in $args) {
        Write-Host "run build for $target start:"
        $res = (Start-Process cargo -ArgumentList "build --release --target x86_64-pc-windows-msvc" -Wait -PassThru -NoNewWindow).ExitCode -Eq 0
        Write-Host "build $target ok: $res"
        $res -and (
            Compress-Archive -WhatIf -Path target/$target/release/fht2p.exe -CompressionLevel Optimal -Force -DestinationPath target/fht2p-$version-$target.zip
        )
    }
}
Function get_version {
    $toml = Get-Content "Cargo.toml"
    $version_line = $toml -match '^version = .+#' 
    Write-Host "version_line: $version_line"
    $script:version = $version_line.Split('"')[1]
    Write-Host "version: $version"
    return $version
}

# (get_version) -and (Write-Host "version2: $version")
(get_version) -and (
    build_and_package "x86_64-pc-windows-msvc" "i586-pc-windows-msvc"
)



