@echo off
rem Abre o Notas Locais numa janela propria do Microsoft Edge (sem barra do navegador), funciona offline.
set "APP=%~dp0src\index.html"
set "APP=%APP:\=/%"
set "PERFIL=%LOCALAPPDATA%\NotasLocais\perfil"
start "" msedge --app="file:///%APP%" --user-data-dir="%PERFIL%" --no-first-run --window-size=1200,800
if errorlevel 1 start "" chrome --app="file:///%APP%" --user-data-dir="%PERFIL%" --no-first-run
