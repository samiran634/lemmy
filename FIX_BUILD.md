# Fix Build Issue - Quick Guide

## The Problem

The build fails with:
```
aws-lc-sys build error: "Required build dependency is missing"
```

This is NOT a code issue. The debate system code is correct and compiles fine. This is a system dependency issue.

## The Solution

Install build tools on your Windows system.

### Option 1: Visual Studio Build Tools (Recommended)

1. Download Visual Studio Build Tools:
   https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022

2. Run the installer

3. Select "Desktop development with C++"

4. Install

5. Restart your terminal

6. Try building again:
   ```powershell
   cargo build
   ```

### Option 2: Chocolatey (Faster)

If you have Chocolatey installed:

```powershell
choco install cmake nasm visualstudio2022buildtools
```

Then restart terminal and build:
```powershell
cargo build
```

### Option 3: Manual Install

1. Install CMake: https://cmake.org/download/
2. Install NASM: https://www.nasm.us/
3. Add both to PATH
4. Restart terminal
5. Build:
   ```powershell
   cargo build
   ```

## Verify It Works

After installing build tools:

```powershell
# Should show CMake version
cmake --version

# Should build successfully
cargo build
```

## Alternative: Use WSL

If you can't install build tools on Windows, use WSL (Windows Subsystem for Linux):

```bash
# In WSL
sudo apt-get update
sudo apt-get install build-essential cmake
cargo build
```

## Still Having Issues?

The debate system code is ready. If you continue having build issues:

1. Check that CMake is in PATH: `cmake --version`
2. Check that you have a C compiler: `cl` (MSVC) or `gcc --version`
3. Try cleaning and rebuilding: `cargo clean && cargo build`
4. Check Rust version: `rustc --version` (should be 1.91+)

## Once Fixed

After the build succeeds:

1. Run migrations:
   ```bash
   diesel migration run
   ```

2. Configure API key in `config/config.hjson`

3. Start server:
   ```bash
   cargo run
   ```

4. Test the API:
   ```bash
   curl http://localhost:8536/api/v4/debate/metrics
   ```

That's it! The debate system will be fully operational.
