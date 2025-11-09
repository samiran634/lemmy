# Simple test script for debate system (PowerShell)

Write-Host "Testing debate system compilation..." -ForegroundColor Green

# Test just the debate crate
Write-Host "`n1. Testing lemmy_debate crate..." -ForegroundColor Yellow
cargo check --package lemmy_debate --features full 2>&1 | Select-String -Pattern "(Finished|error\[E)"

# Test the database schema  
Write-Host "`n2. Testing database schema..." -ForegroundColor Yellow
cargo check --package lemmy_db_schema --features full 2>&1 | Select-String -Pattern "(Finished|error\[E)"

# Test utils
Write-Host "`n3. Testing utils..." -ForegroundColor Yellow
cargo check --package lemmy_utils --features full 2>&1 | Select-String -Pattern "(Finished|error\[E)"

Write-Host "`nDone!" -ForegroundColor Green
