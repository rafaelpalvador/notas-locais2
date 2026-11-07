# Marginália Notes — notas locais em Markdown

## Stack e por quê
- **Tauri 2 + Rust**: instalador de poucos MB, acesso nativo ao disco (leitura/escrita de `.md` sem backend), mesma base para desktop e, depois, Android/iOS.
- **Frontend sem build** (`src/index.html`, JS puro): é o mesmo app do protótipo web. Quando precisar de CodeMirror 6 (WYSIWYG real), migre para Vite + Svelte ou React.
- **Cofre**: `~/Documents/NotasLocais/` — pastas do app = pastas reais; cada nota = `Pasta/Título.md`.

## Estrutura
```
notas-locais/
├─ package.json
├─ src/index.html            # UI (editor, pastas, links, desenho)
└─ src-tauri/
   ├─ tauri.conf.json  Cargo.toml  build.rs
   ├─ capabilities/default.json
   └─ src/main.rs            # comandos: load_vault, save_note, delete_note, create_folder
~/Documents/NotasLocais/
├─ assets/                   # mídias (PDF, áudio, imagens)
├─ Estudo/Juros simples.md
└─ Projetos/Roadmap.md
```

## Esquema da nota
```yaml
---
id: "uuid-v4"
title: "Nome da Nota"
created: 2026-10-25T10:00:00
updated: 2026-10-25T10:30:00
tags: [estudo, tech]
---
```
Corpo em Markdown puro. Desenhos ficam em blocos ```` ```draw ```` com JSON de traços vetoriais. A pasta vem do caminho do arquivo, não do YAML.

## Rodar
1. Instale Rust e os pré-requisitos do Tauri 2 (https://v2.tauri.app/start/prerequisites/).
2. `npm install`
3. `npx tauri icon caminho/para/icone.png` (gera a pasta `src-tauri/icons`)
4. `npm run dev` (ou `npm run build` para o instalador)

Notas sem frontmatter (ex.: vindas do Obsidian) ganham `id` na primeira abertura.

## Próximas fases
1. Persistência nativa + watcher de pasta (`notify`) para refletir edições externas.
2. Mídias: salvar em `assets/` (comando `save_asset`), gravação de áudio com marcadores de tempo.
3. Anotador de PDF (PDF.js empacotado localmente) com camada `.json`.
4. Índice de busca (SQLite FTS5 via `rusqlite`) e criptografia AES-256 por arquivo.
5. Mobile: mover o código para `lib.rs` e rodar `tauri android init` / `tauri ios init`.

## Mídias
Imagens e áudios ficam em `assets/` do cofre e são citados na nota como `![texto](assets/arquivo.png)` e blocos ```` ```audio arquivo.webm ````. Para gravar áudio no macOS, adicione `NSMicrophoneUsageDescription` ao `Info.plist` do app. Arquivos grandes passam pelo IPC como lista de bytes (lento acima de poucos MB); troque por `tauri::ipc::Request` quando precisar.

## PDF
O PDF.js 3.11.174 já está em `src/vendor/pdfjs/` (Apache-2.0), então o anotador de PDF funciona offline no app. As anotações ficam no próprio `.md`, em um bloco ```` ```pdf arquivo.pdf ```` com um JSON por página.

## Fórmulas e incorporações
O KaTeX 0.16.9 está em `src/vendor/katex/` (MIT) e renderiza fórmulas `$...$` e `$$...$$` como MathML, sem precisar de CSS ou fontes. `![[Nota]]` incorpora o conteúdo de outra nota; `![[pasta/Projetos]]` mostra a lista dinâmica das notas da pasta (`![[pasta/Projetos|grade]]` mostra em grade).

## Modo Rico, aparência e transparência
O **Modo leitura** (lê e edita) a nota sem mostrar o Markdown (barra flutuante ao selecionar texto). Em **Aparência** (ícone no topo da barra lateral) há tema, modo transparência (vidro) com ajuste de opacidade, quatro fundos e tamanho do texto. No app desktop, a opção **Janela transparente** deixa ver a área de trabalho através do vidro (`transparent: true` na janela e `macOSPrivateApi` já estão em `tauri.conf.json`; no Linux precisa de compositor).

## Pasta das notas e idioma
- **Pasta local (app desktop):** o ícone de pasta no topo da barra lateral abre o campo para informar o caminho do cofre. O caminho fica salvo em `vault.txt` na pasta de configuração do app; "Restaurar padrão" volta para `Documentos/NotasLocais`. Notas e mídias de outra pasta não são movidas automaticamente.
- **Pasta local (navegador):** no Chrome/Edge do computador, "Escolher pasta…" liga o app a uma pasta real via File System Access API (pasta vazia recebe as notas atuais; pasta com `.md` é carregada).
- **Idioma:** Aparência → Idioma (Português/English). Só a interface é traduzida; o conteúdo das notas não.

## Menções, blocos, propriedades e calendário
- **Menção com `@`:** digite `@` para escolher uma pasta ou nota (ou criar uma nota nova). No arquivo fica `@[Nota]` ou `@[pasta/Nome]`; `!@[Nota]` incorpora o conteúdo. Os formatos antigos `[[Nota]]` e `![[Nota]]` continuam funcionando.
- **Comando `/`:** no início da linha abre os blocos (texto, títulos, checkbox, listas, toggle, chamada, destaque, citação, código, divisor, tabela, fórmula) e inserções (desenho, anexo, áudio, link, incorporação, data, propriedade).
- **Tipo de bloco e tamanho:** ao selecionar texto no Modo leitura, a seta "Texto ▾" troca o tipo do bloco e "Aa ▾" muda o tamanho (`[texto]{.large}`).
- **Propriedades:** o cabeçalho "Propriedades" guarda campos como `author` e `date` no frontmatter YAML do `.md`. Qualquer propriedade com data `AAAA-MM-DD` alimenta o calendário; sem data, vale a data de criação.
- **Calendário e Daily Notes:** o calendário fica na barra lateral; "Nota rápida de hoje" cria ou abre a nota `AAAA-MM-DD` em `Daily Notes`.

## Versão mobile (Android e iOS)
O projeto já tem `src/lib.rs` com o ponto de entrada móvel (`tauri::mobile_entry_point`) e os ícones em `src-tauri/icons`. Nos celulares o cofre padrão fica na pasta de dados do app (`app_data_dir/NotasLocais`).
1. Android: instale o Android Studio (SDK + NDK), defina `ANDROID_HOME` e `NDK_HOME`, depois `npx tauri android init` e `npx tauri android dev`.
2. iOS (somente no macOS): instale o Xcode, depois `npx tauri ios init` e `npx tauri ios dev`.
3. Gravação de áudio no Android pede a permissão `RECORD_AUDIO` no `AndroidManifest.xml` gerado em `src-tauri/gen/android`.
Na versão mobile a interface é a mesma, já responsiva: a barra lateral vira gaveta e, no PDF, o dedo rola e a caneta desenha (ou ligue "Dedo: anotar").

## Verificação
`cargo check` passou (Rust 1.91, Tauri 2) e o formato do frontmatter (propriedades, aspas, nomes de arquivo) tem testes que passaram. Não foi feito build completo com interface nem build Android/iOS.

## Temas, backup, lixeira e pastas
- **Temas:** Aparência → Tema. Além de Auto, Claro e Escuro há Proton, Carbon, Snow, Monokai, Contraste, Calendar e VPN (paletas inspiradas no Proton; não usam logotipos nem fontes da marca). O fundo "Proton" vale para o modo vidro.
- **Backup:** ícone de arquivo no topo da barra lateral → "Exportar tudo (.zip)" gera um zip com as notas em suas pastas (`.md` com frontmatter) e os arquivos de `assets/` citados.
- **Importar:** arquivos `.md` soltos (vão para a pasta "Importadas"), uma pasta inteira ou um `.zip` (inclusive de outros apps). Notas com o mesmo `id` são atualizadas, não duplicadas.
- **Lixeira:** notas e pastas excluídas vão para a lixeira (até 300 itens, guardada no navegador do aparelho) e há "Desfazer" logo após excluir.
- **Pastas:** o botão ⋯ de cada pasta renomeia (atualizando os links `@[pasta/...]`) ou exclui; arraste uma nota da lista, dos Recentes ou dos Favoritos até uma pasta para movê-la. No app desktop há os comandos `rename_folder` e `delete_folder`.
- **Favoritos, tags e modelos:** estrela no título da nota, lista de tags na barra lateral e modelos (menu ▾ ao lado do +), incluindo "Salvar nota atual como modelo".

## Abas, subpastas e tabelas
- **Três abas:** *Markdown* (editor com a visualização ao lado, que pode ser minimizada), *Escrita* (sem Markdown, com a barra de formatação sob o título, também minimizável) e *Leitura* (somente leitura: título e propriedades travados, links e pastas clicáveis).
- **Subpastas e ordem:** arraste uma pasta sobre outra para torná-la subpasta; solte na borda de cima ou de baixo para reordenar. O botão "Editar" ao lado de *Pastas* mostra ↑ ↓ → ← em cada pasta (útil no celular). A ordem fica salva no navegador/aparelho.
- **Relações e metadados:** ocultos por padrão; Aparência permite mostrar "relações no fim da nota" e "metadados (YAML)".
- **Tabelas:** menu + → Tabela (ou `/tabela`). O editor define linhas, colunas, cabeçalho, listras, bordas, espaçamento, cor do cabeçalho e de cada coluna, alinhamento e largura das colunas (arraste as bordas). No arquivo a tabela continua sendo Markdown padrão, com uma linha `<!--tbl {...}-->` logo abaixo para guardar o estilo.

## Configurações da nota, página, capa e cores
- **Menu ☰ (canto direito):** formato da página (Livre, Carta, A4, Caderno), orientação, fundo (liso, pautado, quadriculado, pontilhado), margens, fonte, largura total, capa, imprimir e as ações de copiar/exportar/excluir. As escolhas são por nota e ficam no frontmatter (`page`, `orient`, `pagebg`, `margin`, `font`, `fullw`, `cover`, `coverpos`).
- **Impressão:** o formato escolhido vira o `@page` da impressão; no app desktop o mesmo vale para o PDF exportado (o PDF próprio usa Carta/A4/A5 e paisagem).
- **Capas:** gradientes, cores sólidas ou imagem enviada (salva em `assets/`), com ajuste de posição vertical.
- **Tamanho do texto:** em números (6 a 200 px), gravado no arquivo como `[texto]{size=18}`.
- **Cor das pastas:** menu ⋯ da pasta; fica salva no aparelho junto com a ordem das pastas.

## Novidades (folha infinita, PDF, bases de dados, grafo)
- **Barra lateral recolhível:** o ☰ no topo da barra lateral a esconde; o ☰ no topo da nota a traz de volta. Atalho: Ctrl/Cmd + \.
- **Folha infinita:** Inserir → *Folha infinita* (ou o botão *Folha infinita* em qualquer desenho). Use *Mão*, a barra de espaço ou o botão do meio do mouse para mover, Ctrl/Cmd + roda para zoom, dois dedos no celular. *Tela cheia* ocupa a janela toda (Esc sai). No `.md` fica ```` ```draw dot 520 inf ````; as coordenadas podem ser negativas.
- **Fundos próprios:** no desenho, *Fundo → Enviar fundo…*; na página, ☰ → Fundo da página → *Imagem…*. Aceita imagem, SVG ou PDF (a 1.ª página vira PNG em `assets/`). No `.md`: ```` ```draw img:bg-xxx.png 320 ```` e a propriedade `pagebgimg`.
- **PDF:** a ferramenta *Grifar* agora segue o texto real do PDF (arraste sobre as linhas; um clique grifa a palavra). Em PDFs escaneados, sem texto, ela continua fazendo a linha reta. *Exportar com anotações* gera um PDF novo com traços, grifos e textos (as páginas viram imagem, então o texto do PDF exportado não é selecionável).
- **Bases de dados:** em qualquer lista de notas, alterne *Cartões / Tabela / Quadro*. A tabela mostra as propriedades como colunas editáveis e ordenáveis (+ Propriedade cria uma coluna). O quadro agrupa por qualquer propriedade (padrão `status`); arraste cartões entre colunas, ou clique com o botão direito para mover.
- **Grafo de ligações:** na barra lateral. Mostra as notas e os links `[[…]]`/`@[…]`, com tags e pastas opcionais.

## Novidades (backlinks, versões, Ctrl+K, lembretes, OCR, abas, sincronização)
- **Backlinks com contexto:** o botão `↩ N` ao lado da estrela mostra quais notas citam esta e o trecho de cada citação. Em Aparência → "Mostrar relações", o mesmo aparece no fim da nota.
- **Histórico de versões:** ☰ → *Histórico de versões…*. O app guarda uma versão ao abrir a nota e outra a cada 5 minutos de edição (até 60 por nota, no IndexedDB do aparelho). Mostra o que mudou em relação à versão atual e permite restaurar.
- **Paleta Ctrl/Cmd + K:** busca notas por título e conteúdo; `>` lista os comandos. Enter abre, Ctrl+Enter abre em nova aba, Shift+Enter abre ao lado.
- **Lembretes:** escreva `- [ ] Pagar conta @2026-11-10` ou `@2026-11-10 14:30`. A data aparece como etiqueta (vermelha se atrasada), entra no calendário e na pasta inteligente *Lembretes*. Com o app aberto, um aviso aparece na hora (sem horário: às 9h). Para avisos do sistema, use o comando "Ativar avisos de lembretes" (no navegador; no app desktop os avisos aparecem dentro do app).
- **Reconhecer texto (OCR):** botão no desenho que transforma a escrita à mão em texto pesquisável, inserido logo abaixo como bloco "Texto reconhecido". Usa o Tesseract.js, baixado da internet na primeira vez (por isso não funciona sem internet nem na visualização do Claude). Funciona melhor com letra de forma.
- **Abas e nota ao lado:** cada nota aberta pode virar uma aba (Ctrl/Cmd+clique, botão + ou Ctrl+Enter na paleta). Alt+clique num link, o menu da aba (botão direito) ou ☰ → *Abrir outra nota ao lado…* mostra uma segunda nota em leitura à direita; "Editar" troca as duas de lugar.
- **Sincronização pela pasta:** em *Pasta de notas*, ative "Verificar mudanças feitas fora do app". Coloque o cofre no iCloud Drive, Google Drive, OneDrive, Dropbox ou Syncthing e o app aplica as mudanças feitas em outros aparelhos (a cada 15 s e ao voltar para a janela). Se a mesma nota mudar nos dois lados, a versão de fora vira uma cópia "(conflito AAAA-MM-DD)" — nada é perdido.

## Tabelas mais fáceis
- **Botão "Tabela"** na barra do modo Escrita (e em Inserir +): passe o mouse na grade para escolher linhas × colunas, ou digite os números. "Mais opções…" abre o editor completo (cores, bordas, alinhamento, largura).
- **Escreva direto nas células** no modo Escrita. Tab vai para a próxima célula, Enter desce (na última linha cria uma nova), as setas ↑↓ navegam.
- **Barra rápida:** clique numa célula para + Linha, + Coluna, − Linha, − Coluna ou Editar….
- No editor completo, os campos de linhas e colunas aceitam números digitados.

## Quadro Kanban e fotos
- **Quadro Kanban independente:** Modelos (▾ ao lado do +) → *Novo quadro Kanban*, Ctrl+K → "Novo quadro Kanban", ou Inserir (+) → *Quadro Kanban* dentro de qualquer nota. Os cartões são do próprio quadro (não viram notas, a menos que você use "Transformar em nota").
  - Adicione cartões, arraste entre colunas e dentro delas, arraste colunas pelo título.
  - Clique num cartão para editar título, descrição, data/hora (vira lembrete e aparece no calendário), cor, coluna e concluído.
  - Menu ⋯ da coluna: renomear, cor, mover, ordenar por data, arquivar concluídos, excluir.
  - Pasta inteligente *Quadros* lista todas as notas com quadro.
  - No `.md` fica legível: ```` ```kanban ```` com `## Coluna` e `- [ ] cartão @2026-11-10 #tag {.green}`; a descrição vai indentada abaixo do cartão.
- **Fotos:** clique numa foto (modos Markdown e Escrita) para redimensionar (alça no canto ou 25/50/75/100%), *Recortar* (livre, 1:1, 4:3, 16:9, 3:4 ou original), *Girar* e *Desfazer recorte*. O recorte cria um arquivo novo em `assets/` e guarda o original. No `.md`: `![legenda|420](assets/foto-crop-xxx.png "orig:foto.png")` — o `|420` é a largura em pixels (mesmo formato do Obsidian).

## Recursos inspirados em outros apps
- **Nota trancada:** ☰ → *Trancar com senha…*. O conteúdo vira um bloco ```` ```locked ```` criptografado (AES-256-GCM, chave PBKDF2 com 250 mil iterações); o título e as propriedades continuam visíveis. Ao trocar de nota, ela tranca de novo. O histórico de versões da nota é apagado ao trancar. **Sem a senha não há recuperação.**
- **Sumário, modo foco e contagem:** "≣ Sumário" abaixo do título; *Modo foco* no ☰ ou Ctrl+Shift+F (Esc sai); palavras e tempo de leitura ao lado.
- **Captura rápida:** Ctrl+Shift+Espaço ou "⚡ Captura rápida" na barra lateral (vai para *Caixa de entrada* ou para a nota de hoje). *Salvar da web…*: cole o conteúdo copiado de uma página e ele vira Markdown na pasta *Web*. Colar uma página no modo Markdown também converte. (Um atalho global do sistema, com o app fechado, exigiria o plugin `global-shortcut` do Tauri — ainda não incluído.)
- **Ícones:** clique no ☺ antes do título. Aparecem na barra lateral, nas abas, listas e grafo (propriedade `icon`).
- **Mermaid:** Inserir (+) → *Diagrama (Mermaid)*. A biblioteca é baixada da internet na primeira vez; para usar offline no app, coloque `mermaid.min.js` em `src/vendor/mermaid/`.
- **Localizar e substituir:** Ctrl+H (ou Ctrl+F na nota), com Aa, expressão regular e "Em todas as notas". Uma versão é salva no histórico antes de substituir.
- **Menções sem link:** o botão ↩ mostra notas que citam o título sem link, com "Ligar".
- **Referência a blocos:** passe o mouse num parágrafo e clique em 🔗 para copiar `[[Nota#^id]]`; com `!` na frente incorpora só aquele bloco. `[[Nota#Título]]` liga/incorpora uma seção.
- **Canvas de notas:** Modelos → *Novo canvas de notas* ou Inserir → *Canvas de notas*. Cartões de notas e de texto, ligados por setas (arraste o ● de um cartão até outro), com zoom, mão e tela cheia. Fica em ```` ```canvas ```` (JSON).
- **Flashcards:** escreva `pergunta :: resposta` numa linha. "Revisar flashcards" na barra lateral usa repetição espaçada (SM-2); o progresso fica neste aparelho.
- **Importar:** Dados → *Importar do Notion, Evernote ou HTML…* — exportação do Notion (.zip com Markdown/CSV), Evernote (.enex, com imagens e listas de tarefas) e páginas .html (por exemplo, de exportadores do Apple Notes). O Samsung Notes não exporta em formato aberto; exporte como PDF e anexe.
- **Escanear documento:** Inserir → *Escanear documento*. No celular abre a câmera; filtros Documento (P&B), Realçado e Cinza, giro, e opção de reconhecer o texto (OCR).

## Como testar no Windows (desktop + Android)
1. `powershell -ExecutionPolicy Bypass -File scripts\preparar-windows.ps1` → mostra o que falta e configura JAVA_HOME, ANDROID_HOME, NDK_HOME e os alvos Rust.
2. `npm install` e depois `npm run dev` → abre o app desktop. `npm run build` gera o instalador em `src-tauri\target\release\bundle\`.
3. Android: `npm run android:init`, depois `powershell -ExecutionPolicy Bypass -File scripts\permissoes-android.ps1`, conecte o celular com **Depuração USB** e rode `npm run android:dev`.

## Usar no Windows agora (sem compilar)
Dê dois cliques em **Abrir Notas Locais.bat**: o app abre numa janela própria do Edge, offline. Para ter um ícone na Área de Trabalho e no Menu Iniciar: `powershell -ExecutionPolicy Bypass -File scripts\criar-atalho-windows.ps1`. Use o ícone de pasta → "Escolher pasta" para gravar os `.md` em `Documentos\NotasLocais` (senão as notas ficam no armazenamento do Edge).

## APK e instalador compilados pelo GitHub (sem instalar nada)
O arquivo `.github/workflows/compilar.yml` compila na nuvem o instalador do Windows (`.exe`/`.msi`) e o APK do Android (assinado com chave de teste, para instalar no seu celular). Envie a pasta para um repositório no GitHub, abra a aba **Actions** e baixe os arquivos em **Artifacts** quando terminar.

## Editor estilo Obsidian e explorador em árvore
- O editor agora é o CodeMirror 6 (`src/vendor/cm/cm.min.js`, gerado pela ação "vendor-codemirror" a partir de `tools/cm`).
- **Escrita** (padrão) = visualização ao vivo, como no Obsidian: a marcação (`#`, `**`, `[[ ]]`) some fora da linha do cursor; tabelas, desenhos, PDF, kanban, Mermaid, fórmulas `$$` e embeds aparecem renderizados, com o botão `</>` para editar o texto do bloco.
- Blocos de código com destaque de sintaxe por linguagem, rótulo da linguagem e botão copiar; callouts `> [!tipo]` com cor e ícone.
- **Markdown** = texto-fonte com destaque; **Leitura** = somente leitura. O modo escolhido fica salvo.
- A seção **Arquivos** da barra lateral é uma árvore: pastas expandem e mostram as notas dentro; `+` cria nota na pasta; arraste notas e pastas; botão direito (ou toque longo) numa nota abre Abrir / Abrir ao lado / Renomear / Duplicar / Excluir.
- Celular: a gaveta lateral agora tem uma camada escura real — tocar fora fecha a gaveta sem mexer na nota; o texto da barra não fica mais selecionável; o fundo do app no Android é opaco (`src-tauri/tauri.android.conf.json`).

## Modos de edição estilo Joplin
- Chave **Markdown | Texto rico** no topo: Markdown = editor de texto-fonte; Texto rico = escrita com visualização ao vivo.
- No Markdown, o botão de layout alterna **Dividido → Só editor → Só visualização** (rolagem sincronizada no dividido).
- Botão **Leitura** separado: abre a nota só para ler e, clicando de novo, volta ao editor anterior.
- Barra de ferramentas com ícones: link, listas, tarefas, título, citação, linha, data/hora e anexo.

## Recursos inspirados no Joplin
- **Sincronização WebDAV** (Nextcloud, ownCloud, Koofr, kDrive, pCloud…) com **criptografia de ponta a ponta** opcional (AES-256-GCM, chave PBKDF2). Barra lateral → "☁ Sincronizar" ou Aparência → Sincronização. Notas em `notes/<id>.json`, anexos em `assets/`, índice `index.json` (com ETag para evitar sobrescrita). Conflitos geram uma cópia "(conflito …)".
- **Notas-tarefa**: botão "☐ Tarefa" no cabeçalho, caixa de concluir, alarme com aviso; pasta inteligente "Tarefas (notas)".
- **Busca com filtros**: `tag:` `pasta:` `title:` `body:` `created:` `updated:` (2026-11, day-7, week, month, year-1) `type:todo|note` `iscompleted:0|1` `fixada:1` `any:1` `-palavra` `"frase exata"`, com busca aproximada quando nada bate.
- **Editor externo** (Windows/macOS/Linux): menu ≡ → "Abrir no editor externo" ou "Abrir no Bloco de Notas"; o app acompanha as mudanças salvas lá.
- **Markdown extra**: notas de rodapé `[^1]`, `H~2~O`, `x^2^`, emojis `:rocket:`, sumário `[toc]`, listas de definição (`Termo` + `: definição`).
- **Seleção múltipla**: Ctrl/Shift+clique (ou toque longo no celular) para mover, marcar tag, fixar, virar tarefa ou excluir várias notas.
- **Ordenação e fixar**: botão de ordenar na seção Arquivos (título, atualização, criação; crescente/decrescente) e "📌 Fixar no topo".

## Configurações, menu das notas e barra lateral móvel
- **⚙ Configurações** (botão no topo da barra lateral ou Ctrl + ,): Aparência, Interface, Editor, Arquivos e links, Barra de ferramentas, Sincronização, Atalhos e Avançado, com busca.
- **Menu da nota** (botão ⋯ ou clique direito em qualquer nota da barra lateral, das abas ou da lista): abrir em nova aba / ao lado, copiar, mover, favoritar, fixar, virar tarefa, mesclar, copiar link, histórico, abrir no app padrão, mostrar na pasta, exportar, renomear, apagar.
- **Blocos móveis**: arraste pela alça ⠿ qualquer bloco da barra lateral; esconda blocos e reorganize os botões da barra de formatação em Configurações.

## Marginália Notes (identidade)
- Nome do app: **Marginália Notes** (antes Lume / Notas Locais). Ícone em `src-tauri/icons/marginalia.svg`: folha com a linha de margem e uma pena âmbar; o logo na barra lateral usa as cores do tema escolhido.
- Temas da marca: **Marginália Papel** (padrão em instalações novas) e **Marginália Noite**; os outros temas continuam disponíveis.
- O identificador do app continua `app.notaslocais`, então atualizar não apaga nem duplica as notas.

## Leitor de PDF
- **+ ▾ → Importar PDF (leitor)** ou Configurações → Backup: cria uma nota só para o PDF.
- Barra própria: páginas, página atual, mão (rolar), caneta, marca-texto, borracha, grifar texto, caixa de texto, cor, desfazer, zoom, favoritar página, configurar páginas e exportar com anotações.
- **Páginas**: organizador com miniaturas — arrastar para reordenar, excluir/restaurar, favoritar, girar, filtro "só favoritas".
- **Configurar páginas**: rolagem vertical ou horizontal (deslizar) e página em retrato/paisagem.
- Ordem, páginas excluídas, rotação e favoritos ficam nas propriedades da nota; o arquivo PDF original não é alterado.

- Cor de pasta personalizada: no ⋯ da pasta, escolha uma cor ou digite `#hex`, `rgb()` ou `hsl()`.
- Tabelas: clique numa célula para editar no lugar (Enter desce, Tab avança, Esc cancela).
