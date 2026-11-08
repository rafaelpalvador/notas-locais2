# Céu — previsão do tempo pessoal

App de tempo para Android (e Windows), feito com o mesmo esquema do Marginália Notes.

## Stack
- **Tauri 2 + Rust**: o mesmo código serve para o APK de Android e o instalador de Windows.
- **Frontend sem build** (`src/index.html`, HTML + JS puro): é aqui que se mexe no visual e nas secções.
- **Rust** (`src-tauri/src/lib.rs`): faz os pedidos à internet (sem limites de CORS) e usa o GPS do telemóvel (plugin `geolocation`).

## Fontes de dados (todas gratuitas, sem chave)
| O quê | Fonte |
|---|---|
| Previsão principal (automática ou ECMWF IFS 9 km) | Open-Meteo |
| Comparação de modelos (ECMWF, ICON, GFS, Météo-France) | Open-Meteo |
| Chuva a cada 15 min, próximas 2 h | Open-Meteo (`minutely_15`) |
| Qualidade do ar e pólen | Open-Meteo Air Quality |
| Avisos amarelo/laranja/vermelho | IPMA (distrito mais próximo) |
| Nome da cidade a partir do GPS | BigDataCloud (reserva: OpenStreetMap Nominatim) |
| Pesquisa de cidades | Open-Meteo Geocoding |

## Como funciona a compilação
Esta pasta vive dentro do repositório do Marginália, mas compila à parte: cada `push` que mexa em `ceu-tempo/` corre `.github/workflows/ceu-tempo.yml`.
1. Compila o APK (assinado com `android-assinatura/ceu-teste.jks`, para as atualizações instalarem por cima) e o instalador de Windows.
2. Publica em **Releases** com a tag `ceu-v0.1.N` e o título `Céu 0.1.N`, sem marcar como "latest" (os links fixos `latest` continuam a ser do Marginália), e apaga só as versões antigas do Céu.

No **Obtainium**: adiciona `https://github.com/rafaelpalvador/notas-locais2` e, nas opções, põe `Céu` em *Filtrar títulos das versões por expressão regular*. Na entrada do Marginália põe `Marginália` no mesmo campo, para cada uma apanhar só o seu APK.

## Estrutura
```
src/index.html                 # todo o app (visual, gráficos, lógica)
src-tauri/
  tauri.conf.json              # nome, identificador, janela
  Cargo.toml                   # dependências Rust
  src/lib.rs                   # http_get + GPS
  capabilities/                # permissões (mobile.json = GPS)
  icons/icon.png               # ícone base (os outros são gerados no Actions)
android-assinatura/            # chave de teste do APK
```

## Testar no computador
1. Instala Rust e os pré-requisitos do Tauri 2.
2. `npm install`
3. `npx tauri icon src-tauri/icons/icon.png`
4. `npm run dev`
