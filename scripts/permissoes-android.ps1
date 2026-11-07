# Adiciona as permissoes de microfone e camera ao app Android (rode depois de "npm run android:init")
$m = "src-tauri\gen\android\app\src\main\AndroidManifest.xml"
if (!(Test-Path $m)) { Write-Host "Rode primeiro: npm run android:init" -ForegroundColor Yellow; exit 1 }
$x = Get-Content $m -Raw
foreach ($p in @("android.permission.RECORD_AUDIO","android.permission.CAMERA","android.permission.MODIFY_AUDIO_SETTINGS")) {
  if ($x -notmatch [regex]::Escape($p)) { $x = $x -replace "(<manifest[^>]*>)", "`$1`n    <uses-permission android:name=`"$p`" />" }
}
Set-Content $m $x -Encoding UTF8
Write-Host "Permissoes adicionadas em $m" -ForegroundColor Green
