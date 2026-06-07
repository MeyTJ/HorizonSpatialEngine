#!/usr/bin/env pwsh
# Generates compile-time sqlx query metadata for offline builds.
Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$DatabaseUrl = if ($env:DATABASE_URL) { $env:DATABASE_URL } else { "postgres://horizon:horizon@localhost:5432/horizon_spatial" }

Write-Host "Starting PostGIS..."
docker compose up -d --wait

Write-Host "Running migrations..."
$env:DATABASE_URL = $DatabaseUrl
cargo sqlx migrate run

Write-Host "Generating offline query cache..."
cargo sqlx prepare --workspace

Write-Host "Done. Commit the .sqlx/ directory."
