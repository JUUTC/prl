@echo off
winget install --silent --accept-package-agreements --accept-source-agreements --id Microsoft.VisualStudio.2022.BuildTools > F:\AI\projects\prl\vsbuildtools.log 2>&1
echo WINGET-EXIT-%ERRORLEVEL%>> F:\AI\projects\prl\vsbuildtools.log
