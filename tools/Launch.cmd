@echo off
cd /d "%~dp0"
echo PRL P1 - placeholder playtest
echo Diagnostic output is written to playtest.log in this folder.
prl.exe > playtest.log 2>&1
if errorlevel 1 (
  echo PRL could not run. See playtest.log for the error.
  pause
)
