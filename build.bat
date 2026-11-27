@echo off
chcp 65001 >nul
setlocal
cd /d "%~dp0"

where npm >nul 2>&1
if errorlevel 1 (
  echo npm was not found. Node.js is required.
  pause
  exit /b 1
)

where cargo >nul 2>&1
if errorlevel 1 (
  echo cargo was not found. Rust is required.
  pause
  exit /b 1
)

echo Building Randop.exe...
call npx tauri build --no-bundle
if errorlevel 1 goto fail

copy /Y "src-tauri\target\release\randop.exe" "%~dp0Randop.exe" >nul
if errorlevel 1 goto fail

echo.
echo Done: %~dp0Randop.exe
echo.
pause
exit /b 0

:fail
echo.
echo Build failed.
echo.
pause
exit /b 1
