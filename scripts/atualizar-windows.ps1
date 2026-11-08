# Marginalia Notes - instala ou atualiza pelo terminal do Windows (PowerShell).
# Uso: marginalia-atualizar            (atualiza se houver versao nova)
#      marginalia-atualizar -Forcar    (reinstala mesmo na mesma versao)
#      marginalia-atualizar -Token     (troca o token salvo)
param([switch]$Forcar, [switch]$Token)
$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'
try { [Net.ServicePointManager]::SecurityProtocol = [Net.ServicePointManager]::SecurityProtocol -bor [Net.SecurityProtocolType]::Tls12 } catch {}

$Repo  = 'rafaelpalvador/notas-locais2'
$Api   = "https://api.github.com/repos/$Repo"
$Asset = 'Marginalia-Windows-instalador.exe'
$Dir   = Join-Path $env:LOCALAPPDATA 'MarginaliaNotes'
$TokF  = Join-Path $Dir 'token.dat'
New-Item -ItemType Directory -Force -Path $Dir | Out-Null

function Ok($m)  { Write-Host "OK  $m" -ForegroundColor Green }
function Inf($m) { Write-Host ".. $m" -ForegroundColor Cyan }
function Fail($m){ Write-Host "ERRO $m" -ForegroundColor Red; exit 1 }

# token: variavel MARGINALIA_TOKEN, ou o salvo (criptografado para o seu usuario do Windows), ou pergunta
$tok = $env:MARGINALIA_TOKEN
if (-not $tok -and -not $Token -and (Test-Path $TokF)) {
  try { $sec = Get-Content $TokF | ConvertTo-SecureString; $tok = [Net.NetworkCredential]::new('', $sec).Password } catch { $tok = $null }
}
if (-not $tok) {
  $sec = Read-Host 'Cole o token do GitHub (somente leitura, o mesmo do Obtainium)' -AsSecureString
  $tok = [Net.NetworkCredential]::new('', $sec).Password
}
$tok = "$tok".Trim()
if (-not $tok) { Fail 'Sem token nao da para baixar do repositorio privado.' }
ConvertTo-SecureString $tok -AsPlainText -Force | ConvertFrom-SecureString | Set-Content -Path $TokF

$H = @{ Authorization = "Bearer $tok"; 'User-Agent' = 'MarginaliaNotes'; 'X-GitHub-Api-Version' = '2022-11-28' }

Inf 'Procurando a versao mais nova...'
try { $rel = Invoke-RestMethod -Uri "$Api/releases/latest" -Headers ($H + @{ Accept = 'application/vnd.github+json' }) }
catch { Fail 'O GitHub recusou o token (confira se ele ainda vale e tem acesso a Contents).' }
$a = $rel.assets | Where-Object { $_.name -eq $Asset } | Select-Object -First 1
if (-not $a) { Fail "Nao achei $Asset na versao mais nova." }
$nova = $rel.tag_name -replace '^v', ''

$keys = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\*', 'HKLM:\Software\Microsoft\Windows\CurrentVersion\Uninstall\*', 'HKLM:\Software\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall\*'
$inst = Get-ItemProperty $keys -ErrorAction SilentlyContinue | Where-Object { $_.DisplayName -like 'Margin*Notes*' } | Select-Object -First 1
$atual = if ($inst) { $inst.DisplayVersion } else { $null }

if ($atual -eq $nova -and -not $Forcar) {
  Ok "Voce ja esta na versao mais nova ($nova)."
} else {
  if ($atual) { Inf "Atualizando de $atual para $nova..." } else { Inf "Instalando a versao $nova..." }
  $exe = Join-Path $env:TEMP 'Marginalia-atualizacao.exe'
  try { Invoke-WebRequest -Uri $a.url -Headers ($H + @{ Accept = 'application/octet-stream' }) -OutFile $exe -UseBasicParsing }
  catch { Fail "O download falhou: $($_.Exception.Message)" }
  if ((Get-Item $exe).Length -lt 100000) { Fail 'O arquivo baixado esta incompleto.' }
  Get-Process -ErrorAction SilentlyContinue | Where-Object { $_.Path -and $_.Path -like '*Marginalia Notes*' } | Stop-Process -Force -ErrorAction SilentlyContinue
  Start-Sleep -Milliseconds 600
  $p = Start-Process -FilePath $exe -ArgumentList '/P', '/R' -PassThru -Wait
  Remove-Item $exe -ErrorAction SilentlyContinue
  if ($p.ExitCode -ne 0) { Fail "O instalador terminou com erro ($($p.ExitCode))." }
  Ok "Marginalia Notes $nova instalado. O app abre sozinho."
}

# guarda este script como o comando "marginalia-atualizar"
try {
  $src = Invoke-RestMethod -Uri "$Api/contents/scripts/atualizar-windows.ps1" -Headers ($H + @{ Accept = 'application/vnd.github.raw' })
  if ($src -match 'Marginalia Notes') {
    [IO.File]::WriteAllText((Join-Path $Dir 'atualizar.ps1'), $src, [Text.UTF8Encoding]::new($true))
    $apps = Join-Path $env:LOCALAPPDATA 'Microsoft\WindowsApps'
    $cmd = "@echo off`r`npowershell -NoProfile -ExecutionPolicy Bypass -File `"%LOCALAPPDATA%\MarginaliaNotes\atualizar.ps1`" %*`r`n"
    [IO.File]::WriteAllText((Join-Path $apps 'marginalia-atualizar.cmd'), $cmd, [Text.Encoding]::ASCII)
  }
} catch {}
Inf 'Para atualizar de novo no futuro, abra o terminal e digite: marginalia-atualizar'
