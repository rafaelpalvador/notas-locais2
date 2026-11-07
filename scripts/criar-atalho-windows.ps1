# Cria um atalho "Notas Locais" na Area de Trabalho e no Menu Iniciar, com o icone do app.
# Uso: powershell -ExecutionPolicy Bypass -File scripts\criar-atalho-windows.ps1
$raiz = Split-Path -Parent $PSScriptRoot
$app = (Join-Path $raiz "src\index.html") -replace '\\','/'
$icone = Join-Path $raiz "src-tauri\icons\icon.ico"
$perfil = Join-Path $env:LOCALAPPDATA "NotasLocais\perfil"
$edge = @("${env:ProgramFiles(x86)}\Microsoft\Edge\Application\msedge.exe","$env:ProgramFiles\Microsoft\Edge\Application\msedge.exe") | Where-Object { Test-Path $_ } | Select-Object -First 1
if (!$edge) { Write-Host "Microsoft Edge nao encontrado." -ForegroundColor Yellow; exit 1 }
$sh = New-Object -ComObject WScript.Shell
foreach ($dest in @([Environment]::GetFolderPath("Desktop"), (Join-Path $env:APPDATA "Microsoft\Windows\Start Menu\Programs"))) {
  $l = $sh.CreateShortcut((Join-Path $dest "Notas Locais.lnk"))
  $l.TargetPath = $edge
  $l.Arguments = "--app=`"file:///$app`" --user-data-dir=`"$perfil`" --no-first-run --window-size=1200,800"
  $l.IconLocation = $icone
  $l.Description = "Notas Locais"
  $l.Save()
}
Write-Host "Atalho criado na Area de Trabalho e no Menu Iniciar." -ForegroundColor Green
