@echo off
setlocal enableextensions
set "DIR=%~dp0"
set "BIN_DIR=%DIR%..\bin"

echo Building dispatch in release mode...
cargo build --release --manifest-path "%DIR%Cargo.toml"
if errorlevel 1 exit /b 1

set "SRC=%DIR%target\release\dispatch.exe"
set "LINK=%BIN_DIR%\dispatch.exe"

if not exist "%SRC%" (
    echo ERROR: built binary not found at %SRC%
    exit /b 1
)

echo Removing stale entries in %BIN_DIR% ...
if exist "%BIN_DIR%\dispatch"     del /f /q "%BIN_DIR%\dispatch"
if exist "%BIN_DIR%\dispatch.exe" del /f /q "%BIN_DIR%\dispatch.exe"

echo Creating symlink %LINK% -^> %SRC%
mklink "%LINK%" "%SRC%"
if errorlevel 1 (
    echo.
    echo mklink failed. On Windows, real symlinks require either:
    echo   - Developer Mode enabled ^(Settings -^> For developers^), or
    echo   - Running this script as Administrator.
    exit /b 1
)

dir "%LINK%"
echo Done.
endlocal
