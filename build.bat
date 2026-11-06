@echo off
chcp 65001 >nul
setlocal
cd /d "%~dp0"

where npm >nul 2>&1
if errorlevel 1 (
  echo Не найден npm. Нужен Node.js.
  pause
  exit /b 1
)

where cargo >nul 2>&1
if errorlevel 1 (
  echo Не найден cargo. Нужен Rust.
  pause
  exit /b 1
)

echo Сборка Randop.exe...
call npx tauri build --no-bundle
if errorlevel 1 goto fail

copy /Y "src-tauri\target\release\randop.exe" "%~dp0Randop.exe" >nul
if errorlevel 1 goto fail

echo.
echo Готово: %~dp0Randop.exe
echo.
pause
exit /b 0

:fail
echo.
echo Сборка не удалась.
echo.
pause
exit /b 1
