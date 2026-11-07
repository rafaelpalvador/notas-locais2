# Notas Locais - verifica e prepara o Windows para compilar (desktop e Android)
# Uso (PowerShell, dentro da pasta do projeto):
#   powershell -ExecutionPolicy Bypass -File scripts\preparar-windows.ps1
$ErrorActionPreference = "Continue"
function Ok($m){ Write-Host "  [ok] $m" -ForegroundColor Green }
function Falta($m){ Write-Host "  [falta] $m" -ForegroundColor Yellow; $script:faltas++ }
$faltas = 0

Write-Host "`n== Desktop ==" -ForegroundColor Cyan
if (Get-Command node -ErrorAction SilentlyContinue) { Ok "Node.js $(node -v)" } else { Falta "Node.js LTS  ->  winget install OpenJS.NodeJS.LTS" }
if (Get-Command cargo -ErrorAction SilentlyContinue) { Ok "Rust $(rustc --version)" } else { Falta "Rust  ->  winget install Rustlang.Rustup   (depois feche e abra o PowerShell)" }
$vswhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
$vc = $null; if (Test-Path $vswhere) { $vc = & $vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath }
if ($vc) { Ok "C++ Build Tools ($vc)" } else { Falta "Microsoft C++ Build Tools com 'Desenvolvimento para desktop com C++'  ->  winget install Microsoft.VisualStudio.2022.BuildTools  (no instalador, marque essa carga de trabalho)" }
$wv = Get-ItemProperty "HKLM:\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}" -ErrorAction SilentlyContinue
if ($wv) { Ok "WebView2 $($wv.pv)" } else { Write-Host "  [?] WebView2 nao encontrado no registro (ja vem no Windows 10/11; se o app nao abrir, instale o 'Evergreen Bootstrapper')" }

Write-Host "`n== Android ==" -ForegroundColor Cyan
$studioJbr = "C:\Program Files\Android\Android Studio\jbr"
$sdk = "$env:LOCALAPPDATA\Android\Sdk"
if (Test-Path $studioJbr) {
  if ($env:JAVA_HOME -ne $studioJbr) { [Environment]::SetEnvironmentVariable("JAVA_HOME", $studioJbr, "User"); $env:JAVA_HOME = $studioJbr; Ok "JAVA_HOME definido: $studioJbr" } else { Ok "JAVA_HOME" }
} else { Falta "Android Studio  ->  https://developer.android.com/studio  (instale e abra uma vez)" }
if (Test-Path $sdk) {
  [Environment]::SetEnvironmentVariable("ANDROID_HOME", $sdk, "User"); $env:ANDROID_HOME = $sdk; Ok "ANDROID_HOME: $sdk"
  $ndkDir = Join-Path $sdk "ndk"
  if (Test-Path $ndkDir) { $v = Get-ChildItem -Name $ndkDir | Select-Object -Last 1; $ndk = Join-Path $ndkDir $v; [Environment]::SetEnvironmentVariable("NDK_HOME", $ndk, "User"); $env:NDK_HOME = $ndk; Ok "NDK_HOME: $ndk" }
  else { Falta "NDK  ->  Android Studio > Settings > Android SDK > SDK Tools > marque 'NDK (Side by side)'" }
  foreach ($p in @("platform-tools","build-tools","platforms","cmdline-tools")) { if (Test-Path (Join-Path $sdk $p)) { Ok "SDK: $p" } else { Falta "SDK: $p  ->  instale pelo SDK Manager do Android Studio" } }
  $pt = Join-Path $sdk "platform-tools"; if ((Test-Path $pt) -and ($env:Path -notlike "*$pt*")) { $up=[Environment]::GetEnvironmentVariable("Path","User"); [Environment]::SetEnvironmentVariable("Path", "$up;$pt", "User"); $env:Path += ";$pt"; Ok "adb adicionado ao PATH" }
} else { Falta "Android SDK nao encontrado em $sdk (abra o Android Studio e conclua a instalacao)" }
if (Get-Command rustup -ErrorAction SilentlyContinue) { rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android | Out-Null; Ok "alvos Rust do Android" }
$dev = Get-ItemProperty "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\AppModelUnlock" -ErrorAction SilentlyContinue
if ($dev -and $dev.AllowDevelopmentWithoutDevLicense -eq 1) { Ok "Modo de desenvolvedor do Windows" } else { Write-Host "  [dica] Se o build Android falhar com 'privilegio necessario'/symlink, ative: Configuracoes > Sistema > Para desenvolvedores > Modo de desenvolvedor" }
if (Get-Command adb -ErrorAction SilentlyContinue) { $d = (adb devices) -join " "; if ($d -match "\tdevice") { Ok "celular conectado via USB" } else { Write-Host "  [i] nenhum celular conectado (ative 'Depuracao USB' e aceite no celular)" } }

Write-Host ""
if ($faltas -eq 0) { Write-Host "Tudo pronto. Feche e reabra o PowerShell e siga o passo a passo." -ForegroundColor Green } else { Write-Host "$faltas item(ns) faltando. Instale-os e rode este script de novo." -ForegroundColor Yellow }
