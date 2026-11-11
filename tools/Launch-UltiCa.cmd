@echo off
cd /d "%~dp0"
echo PRL - UltiCa crypt candidate (fixed player appearance, not final art)
echo Diagnostic output is written to playtest-ultica.log in this folder.
prl.exe --tileset=ultica > playtest-ultica.log 2>&1
if errorlevel 1 (
  echo PRL could not run. See playtest-ultica.log for the error.
  pause
)
