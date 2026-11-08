#!/usr/bin/env bash
# Marginália Notes — instala ou atualiza pelo terminal (qualquer distribuição Linux).
# Uso: marginalia-atualizar            (atualiza se houver versão nova)
#      marginalia-atualizar --forcar   (reinstala mesmo na mesma versão)
#      marginalia-atualizar --token    (troca o token salvo)
#      marginalia-atualizar --remover  (desinstala; as notas ficam onde estão)
set -euo pipefail

REPO="rafaelpalvador/notas-locais2"
API="https://api.github.com/repos/$REPO"
ASSET="Marginalia-Linux.AppImage"
CFG="${XDG_CONFIG_HOME:-$HOME/.config}/marginalia-notes"
APPDIR="${XDG_DATA_HOME:-$HOME/.local/share}/marginalia-notes"
BIN="$HOME/.local/bin"
DESK="${XDG_DATA_HOME:-$HOME/.local/share}/applications/marginalia-notes.desktop"

FORCAR=0; NOVOTOKEN=0; REMOVER=0
for a in "$@"; do case "$a" in
  -f|--forcar|--force) FORCAR=1;;
  --token) NOVOTOKEN=1;;
  --remover|--uninstall) REMOVER=1;;
  -h|--help) sed -n '2,7p' "$0" 2>/dev/null || true; exit 0;;
esac; done

ok()  { printf '\033[1;32m✔\033[0m %s\n' "$*"; }
inf() { printf '\033[1;34m•\033[0m %s\n' "$*"; }
err() { printf '\033[1;31m✘\033[0m %s\n' "$*" >&2; exit 1; }

if [ "$REMOVER" = 1 ]; then
  pkill -f "$APPDIR/app/" 2>/dev/null || true
  rm -rf "$APPDIR" "$DESK" "$BIN/marginalia-notes" "$BIN/marginalia-atualizar"
  ok "Marginália Notes removido. Suas notas não foram apagadas."
  exit 0
fi

command -v curl >/dev/null || err "Instale o curl (openSUSE: sudo zypper in curl · Pop!_OS: sudo apt install curl · CachyOS: sudo pacman -S curl)"

mkdir -p "$CFG" "$APPDIR" "$BIN" "$(dirname "$DESK")"; chmod 700 "$CFG"
TOKF="$CFG/token"
TOK="${MARGINALIA_TOKEN:-}"
[ "$NOVOTOKEN" = 1 ] && TOK=""
if [ -z "$TOK" ] && [ "$NOVOTOKEN" = 0 ] && [ -f "$TOKF" ]; then TOK="$(cat "$TOKF")"; fi
if [ -z "$TOK" ]; then
  [ -r /dev/tty ] || err "Defina MARGINALIA_TOKEN com o token do GitHub."
  printf 'Cole o token do GitHub (somente leitura, o mesmo do Obtainium): ' >/dev/tty
  IFS= read -rs TOK </dev/tty; echo >/dev/tty
fi
TOK="$(printf %s "$TOK" | tr -d '[:space:]')"
[ -n "$TOK" ] || err "Sem token não dá para baixar do repositório privado."
( umask 077; printf %s "$TOK" > "$TOKF" )

H=(-H "Authorization: Bearer $TOK" -H "User-Agent: MarginaliaNotes" -H "X-GitHub-Api-Version: 2022-11-28")

inf "Procurando a versão mais nova…"
J="$(curl -fsSL "${H[@]}" -H "Accept: application/vnd.github+json" "$API/releases/latest")" || err "O GitHub recusou o token (confira se ele ainda vale e tem acesso a Contents)."
TAG="$(printf %s "$J" | grep -o '"tag_name": *"[^"]*"' | head -1 | sed 's/.*"\([^"]*\)"$/\1/')"
URL="$(printf %s "$J" | grep -o '"url": *"[^"]*/releases/assets/[0-9]*"\|"name": *"[^"]*"' | awk -v want="$ASSET" '
  /"url"/ { gsub(/.*"url": *"|"$/, ""); u=$0; next }
  /"name"/ { gsub(/.*"name": *"|"$/, ""); if ($0==want && u!="") { print u; exit } }')"
[ -n "$TAG" ] && [ -n "$URL" ] || err "Não achei $ASSET na versão mais nova."

ATUAL="$(cat "$APPDIR/versao" 2>/dev/null || true)"
if [ "$ATUAL" = "$TAG" ] && [ "$FORCAR" = 0 ] && [ -x "$APPDIR/app/AppRun" ]; then
  ok "Você já está na versão mais nova (${TAG#v})."
else
  TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
  inf "Baixando ${TAG#v}…"
  curl -fL --progress-bar "${H[@]}" -H "Accept: application/octet-stream" -o "$TMP/m.AppImage" "$URL" || err "O download falhou."
  [ "$(head -c4 "$TMP/m.AppImage" | od -An -tx1 | tr -d ' \n')" = "7f454c46" ] || err "O arquivo baixado não é um AppImage."
  chmod +x "$TMP/m.AppImage"
  inf "Instalando…"
  ( cd "$TMP" && ./m.AppImage --appimage-extract >/dev/null ) || err "Não consegui abrir o AppImage."
  pkill -f "$APPDIR/app/" 2>/dev/null || true; sleep 0.5
  rm -rf "$APPDIR/app.old"; [ -d "$APPDIR/app" ] && mv "$APPDIR/app" "$APPDIR/app.old"
  mv "$TMP/squashfs-root" "$APPDIR/app"; rm -rf "$APPDIR/app.old"
  printf %s "$TAG" > "$APPDIR/versao"
  ICON="$(find "$APPDIR/app" -maxdepth 1 -name '*.png' | head -1)"; [ -n "$ICON" ] && cp -f "$ICON" "$APPDIR/icone.png"
  cat > "$BIN/marginalia-notes" <<EOS
#!/bin/sh
# evita janela em branco em algumas placas de vídeo / Wayland
export WEBKIT_DISABLE_DMABUF_RENDERER=\${WEBKIT_DISABLE_DMABUF_RENDERER:-1}
exec "$APPDIR/app/AppRun" "\$@"
EOS
  chmod +x "$BIN/marginalia-notes"
  cat > "$DESK" <<EOS
[Desktop Entry]
Type=Application
Name=Marginália Notes
Comment=Notas em Markdown, locais e privadas
Exec=$BIN/marginalia-notes %U
Icon=$APPDIR/icone.png
Terminal=false
Categories=Office;Utility;TextEditor;
StartupWMClass=marginalia-notes
EOS
  command -v update-desktop-database >/dev/null && update-desktop-database "$(dirname "$DESK")" >/dev/null 2>&1 || true
  ok "Marginália Notes ${TAG#v} instalado. Abra pelo menu de aplicativos ou com: marginalia-notes"
fi

# guarda este script como o comando "marginalia-atualizar"
if curl -fsSL "${H[@]}" -H "Accept: application/vnd.github.raw" "$API/contents/scripts/atualizar-linux.sh" -o "$BIN/.marginalia-atualizar.novo" 2>/dev/null && head -1 "$BIN/.marginalia-atualizar.novo" | grep -q bash; then
  mv -f "$BIN/.marginalia-atualizar.novo" "$BIN/marginalia-atualizar"; chmod +x "$BIN/marginalia-atualizar"
else rm -f "$BIN/.marginalia-atualizar.novo"; fi
case ":$PATH:" in *":$BIN:"*) ;; *) inf "Adicione $BIN ao PATH para usar os comandos (ex.: echo 'export PATH=\"\$HOME/.local/bin:\$PATH\"' >> ~/.bashrc)";; esac
inf "Para atualizar de novo no futuro: marginalia-atualizar"
