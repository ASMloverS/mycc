@echo off
setlocal enableextensions
set "DIR=%~dp0"
set "BIN_DIR=%DIR%.."
set "TOOLS_DIR=%DIR%tools"
set "DISPATCH_BIN_DIR=%DIR%bin"

echo Building dispatch and harness-install in release mode...
cargo build --release --bin dispatch --bin harness-install --manifest-path "%DIR%Cargo.toml"
if errorlevel 1 exit /b 1

:: Deploy dispatch (symlink in ../bin/)
set "SRC_DISPATCH=%DIR%target\release\dispatch.exe"
set "LINK_DISPATCH=%BIN_DIR%\dispatch.exe"

if not exist "%SRC_DISPATCH%" (
    echo ERROR: built binary not found at %SRC_DISPATCH%
    exit /b 1
)

echo Removing stale dispatch entries in %BIN_DIR% ...
if exist "%BIN_DIR%\dispatch"     del /f /q "%BIN_DIR%\dispatch"
if exist "%BIN_DIR%\dispatch.exe" del /f /q "%BIN_DIR%\dispatch.exe"

echo Creating symlink %LINK_DISPATCH% -^> %SRC_DISPATCH%
mklink "%LINK_DISPATCH%" "%SRC_DISPATCH%"
if errorlevel 1 (
    echo.
    echo mklink failed. On Windows, real symlinks require either:
    echo   - Developer Mode enabled ^(Settings -^> For developers^), or
    echo   - Running this script as Administrator.
    exit /b 1
)

:: Deploy harness-install to dispatch-rs/bin/
set "SRC_INSTALL=%DIR%target\release\harness-install.exe"
set "DEST_INSTALL=%DISPATCH_BIN_DIR%\harness-install.exe"

if not exist "%SRC_INSTALL%" (
    echo ERROR: built binary not found at %SRC_INSTALL%
    exit /b 1
)

if not exist "%DISPATCH_BIN_DIR%" mkdir "%DISPATCH_BIN_DIR%"

echo Copying %SRC_INSTALL% -^> %DEST_INSTALL%
copy /y "%SRC_INSTALL%" "%DEST_INSTALL%"
if errorlevel 1 exit /b 1

echo Done.
dir "%LINK_DISPATCH%"
dir "%DEST_INSTALL%"

endlocal
