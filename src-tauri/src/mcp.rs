// Marginália Notes como conector do Claude (MCP pela entrada/saída padrão).
// Uso: "Marginalia Notes" --mcp   (o Claude abre este processo e conversa por JSON-RPC)
use serde_json::{json, Value};
use std::{fs, io::{BufRead, Write}, path::{Path, PathBuf}};

use crate::{find, render, safe, walk, Note};

const APP_ID: &str = "app.notaslocais";

fn home() -> Option<PathBuf> { std::env::var_os(if cfg!(windows) { "USERPROFILE" } else { "HOME" }).map(PathBuf::from) }

fn config_dir() -> Option<PathBuf> {
    if cfg!(windows) { std::env::var_os("APPDATA").map(PathBuf::from) }
    else if cfg!(target_os = "macos") { home().map(|h| h.join("Library/Application Support")) }
    else { std::env::var_os("XDG_CONFIG_HOME").map(PathBuf::from).or_else(|| home().map(|h| h.join(".config"))) }
}
fn data_dir() -> Option<PathBuf> {
    if cfg!(windows) { std::env::var_os("APPDATA").map(PathBuf::from) }
    else if cfg!(target_os = "macos") { home().map(|h| h.join("Library/Application Support")) }
    else { std::env::var_os("XDG_DATA_HOME").map(PathBuf::from).or_else(|| home().map(|h| h.join(".local/share"))) }
}
fn documents_dir() -> Option<PathBuf> {
    static D: std::sync::OnceLock<Option<PathBuf>> = std::sync::OnceLock::new();
    D.get_or_init(documents_dir_raw).clone()
}
fn documents_dir_raw() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        // pasta Documentos de verdade (pode ter sido movida, ex.: para o OneDrive)
        use std::os::windows::process::CommandExt;
        if let Ok(out) = std::process::Command::new("powershell").args(["-NoProfile", "-Command", "[Environment]::GetFolderPath('MyDocuments')"]).creation_flags(0x0800_0000).output() {
            let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !s.is_empty() && Path::new(&s).is_dir() { return Some(PathBuf::from(s)); }
        }
        home().map(|h| h.join("Documents"))
    }
    #[cfg(not(windows))]
    {
        if let Some(h) = home() {
            let cfgf = std::env::var_os("XDG_CONFIG_HOME").map(PathBuf::from).unwrap_or_else(|| h.join(".config")).join("user-dirs.dirs");
            if let Ok(t) = fs::read_to_string(cfgf) {
                for l in t.lines() {
                    if let Some(v) = l.strip_prefix("XDG_DOCUMENTS_DIR=") {
                        let v = v.trim_matches('"').replace("$HOME", &h.to_string_lossy());
                        if Path::new(&v).is_dir() { return Some(PathBuf::from(v)); }
                    }
                }
            }
            let d = h.join("Documents");
            if d.is_dir() { return Some(d); }
        }
        data_dir().map(|d| d.join(APP_ID))
    }
}
/// a mesma pasta que o app usa (vault.txt escolhido nas configurações, senão Documentos/NotasLocais)
pub fn vault_root() -> Result<PathBuf, String> {
    if let Some(c) = config_dir() {
        if let Ok(s) = fs::read_to_string(c.join(APP_ID).join("vault.txt")) {
            let s = s.trim();
            if !s.is_empty() { return Ok(PathBuf::from(s)); }
        }
    }
    documents_dir().map(|d| d.join("NotasLocais")).ok_or_else(|| "Não achei a pasta das notas".into())
}

// ---------- utilidades ----------
fn now_iso() -> String {
    let secs = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0);
    let (days, rem) = (secs.div_euclid(86400), secs.rem_euclid(86400));
    // dias desde 1970 -> data civil (algoritmo de Howard Hinnant)
    let z = days + 719468; let era = z.div_euclid(146097); let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365; let y = yoe + era * 400; let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153; let d = doy - (153 * mp + 2) / 5 + 1; let m = if mp < 10 { mp + 3 } else { mp - 9 }; let y = if m <= 2 { y + 1 } else { y };
    format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.000Z", y, m, d, rem / 3600, rem % 3600 / 60, rem % 60)
}
fn new_id() -> String {
    use std::hash::{BuildHasher, Hasher};
    let mut out = String::new();
    for i in 0..2u64 {
        let mut h = std::collections::hash_map::RandomState::new().build_hasher();
        h.write_u128(std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0));
        h.write_u64(i); h.write_u32(std::process::id());
        out.push_str(&format!("{:016x}", h.finish()));
    }
    format!("{}-{}-4{}-a{}-{}", &out[0..8], &out[8..12], &out[13..16], &out[17..20], &out[20..32])
}
fn fold(s: &str) -> String {
    s.chars().flat_map(|c| c.to_lowercase()).map(|c| match c {
        'á' | 'à' | 'â' | 'ã' | 'ä' => 'a', 'é' | 'è' | 'ê' | 'ë' => 'e', 'í' | 'ì' | 'î' | 'ï' => 'i',
        'ó' | 'ò' | 'ô' | 'õ' | 'ö' => 'o', 'ú' | 'ù' | 'û' | 'ü' => 'u', 'ç' => 'c', 'ñ' => 'n', x => x,
    }).collect()
}
fn locked(n: &Note) -> bool { n.body.starts_with("```locked\n") }
const HIDDEN: &[&str] = &["todo", "fixada", "alarme", "tipo", "pdfOrd", "pdfDel", "pdfRot", "pdfFav", "pdfLay", "banco"];

fn load() -> Result<(PathBuf, Vec<Note>, Vec<String>), String> {
    let r = vault_root()?;
    if !r.is_dir() { return Err(format!("A pasta das notas não existe: {}", r.display())); }
    let (mut notes, mut folders) = (vec![], vec![]);
    walk(&r, &r, &mut notes, &mut folders);
    folders.sort();
    notes.sort_by(|a, b| b.updated.cmp(&a.updated));
    Ok((r, notes, folders))
}
fn pick<'a>(notes: &'a [Note], a: &Value) -> Result<&'a Note, String> {
    if let Some(id) = a.get("id").and_then(|v| v.as_str()).filter(|s| !s.is_empty()) {
        return notes.iter().find(|n| n.id == id).ok_or_else(|| format!("Nenhuma nota com id {id}"));
    }
    let t = a.get("title").and_then(|v| v.as_str()).unwrap_or("").trim();
    if t.is_empty() { return Err("Informe o id ou o título da nota".into()); }
    let ft = fold(t);
    let exact: Vec<&Note> = notes.iter().filter(|n| fold(&n.title) == ft).collect();
    if exact.len() == 1 { return Ok(exact[0]); }
    if exact.len() > 1 { return Err(format!("Há {} notas com o título “{t}”; use o id: {}", exact.len(), exact.iter().map(|n| format!("{} ({})", n.id, if n.folder.is_empty() { "raiz" } else { n.folder.as_str() })).collect::<Vec<_>>().join(", "))); }
    let part: Vec<&Note> = notes.iter().filter(|n| fold(&n.title).contains(&ft)).collect();
    if part.len() == 1 { return Ok(part[0]); }
    if part.is_empty() { Err(format!("Não achei nota com o título “{t}”")) } else { Err(format!("Vários títulos parecidos com “{t}”: {}", part.iter().take(10).map(|n| format!("“{}” (id {})", n.title, n.id)).collect::<Vec<_>>().join(", "))) }
}
fn save(root: &Path, note: &Note) -> Result<(), String> {
    let mut dir = root.to_path_buf();
    for s in note.folder.split('/').filter(|s| !s.is_empty() && *s != "..") { dir.push(safe(s)); }
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let old = find(root, &note.id);
    let mut path = dir.join(format!("{}.md", safe(&note.title)));
    if path.exists() && old.as_ref() != Some(&path) { path = dir.join(format!("{} ({}).md", safe(&note.title), &note.id[..note.id.len().min(4)])); }
    fs::write(&path, render(note)).map_err(|e| e.to_string())?;
    if let Some(o) = old { if o != path { let _ = fs::remove_file(o); } }
    Ok(())
}
fn meta(n: &Note) -> String {
    format!("• {} — pasta: {} — id: {} — atualizada: {}", n.title, if n.folder.is_empty() { "(raiz)" } else { n.folder.as_str() }, n.id, n.updated.get(..10).unwrap_or(&n.updated))
}
fn full(n: &Note) -> String {
    let props: Vec<String> = n.props.iter().filter(|(k, _)| !HIDDEN.contains(&k.as_str())).map(|(k, v)| format!("{k}: {v}")).collect();
    let mut s = format!("# {}\nid: {} · pasta: {} · criada: {} · atualizada: {}\n", n.title, n.id, if n.folder.is_empty() { "(raiz)" } else { n.folder.as_str() }, n.created, n.updated);
    if let Some(t) = n.props.get("todo") { s.push_str(if t == "x" { "tarefa: concluída\n" } else { "tarefa: pendente\n" }); }
    if let Some(a) = n.props.get("alarme") { s.push_str(&format!("lembrete: {a}\n")); }
    if !props.is_empty() { s.push_str(&format!("propriedades: {}\n", props.join(" · "))); }
    s.push_str("\n---\n\n");
    if locked(n) { s.push_str("(Nota protegida com senha — o conteúdo não pode ser lido aqui.)"); } else { s.push_str(&n.body); }
    s
}

// ---------- ferramentas ----------
fn tools() -> Value {
    let s = |p: Value, req: &[&str]| json!({"type":"object","properties":p,"required":req});
    let idt = json!({"id":{"type":"string","description":"id da nota (preferível)"},"title":{"type":"string","description":"título da nota, se não souber o id"}});
    let mut idt_text = idt.clone(); idt_text["text"] = json!({"type":"string","description":"texto em Markdown"});
    let mut idt_rep = idt.clone(); idt_rep["old"] = json!({"type":"string","description":"trecho exato que existe hoje na nota"}); idt_rep["new"] = json!({"type":"string","description":"texto que entra no lugar"});
    let mut idt_upd = idt.clone(); idt_upd["text"] = json!({"type":"string","description":"novo conteúdo completo (substitui o atual)"}); idt_upd["new_title"] = json!({"type":"string"}); idt_upd["folder"] = json!({"type":"string","description":"mover para esta pasta (ex.: Estudo/Tech); vazio = raiz"});
    json!([
      {"name":"list_folders","description":"Lista as pastas do Marginália Notes e quantas notas cada uma tem.","inputSchema":s(json!({}),&[])},
      {"name":"list_notes","description":"Lista notas (mais recentes primeiro), opcionalmente de uma pasta ou com uma tag (#tag).","inputSchema":s(json!({"folder":{"type":"string"},"tag":{"type":"string"},"limit":{"type":"integer","default":50}}),&[])},
      {"name":"search_notes","description":"Busca palavras no título e no texto das notas (ignora acentos e maiúsculas). Devolve título, pasta, id e um trecho.","inputSchema":s(json!({"query":{"type":"string"},"folder":{"type":"string"},"limit":{"type":"integer","default":20}}),&["query"])},
      {"name":"read_note","description":"Lê uma nota inteira (Markdown) com pasta, datas e propriedades.","inputSchema":s(idt.clone(),&[])},
      {"name":"create_note","description":"Cria uma nota nova em Markdown. A primeira linha não precisa repetir o título.","inputSchema":s(json!({"title":{"type":"string"},"text":{"type":"string","description":"conteúdo em Markdown"},"folder":{"type":"string","description":"pasta (ex.: Estudo/Tech); vazio = raiz"}}),&["title","text"])},
      {"name":"append_to_note","description":"Acrescenta texto no fim de uma nota existente.","inputSchema":s(idt_text,&["text"])},
      {"name":"replace_in_note","description":"Troca um trecho exato de uma nota por outro (use read_note antes para copiar o trecho).","inputSchema":s(idt_rep,&["old","new"])},
      {"name":"update_note","description":"Reescreve o conteúdo, renomeia ou move uma nota. Só os campos enviados mudam.","inputSchema":s(idt_upd,&[])},
      {"name":"list_tasks","description":"Lista as tarefas (- [ ]) das notas, com a nota de origem. Por padrão só as pendentes.","inputSchema":s(json!({"folder":{"type":"string"},"include_done":{"type":"boolean","default":false}}),&[])}
    ])
}
fn in_folder(n: &Note, f: Option<&str>) -> bool {
    match f.map(|x| x.trim_matches('/')).filter(|x| !x.is_empty()) { None => true, Some(f) => n.folder == f || n.folder.starts_with(&format!("{f}/")) }
}
fn call(name: &str, a: &Value) -> Result<String, String> {
    let gs = |k: &str| a.get(k).and_then(|v| v.as_str());
    let gi = |k: &str, d: usize| a.get(k).and_then(|v| v.as_u64()).map(|x| x as usize).unwrap_or(d).clamp(1, 500);
    let (root, notes, folders) = load()?;
    match name {
        "list_folders" => {
            let mut out = format!("Pasta das notas: {}\n{} notas no total.\n", root.display(), notes.len());
            let raiz = notes.iter().filter(|n| n.folder.is_empty()).count();
            out.push_str(&format!("• (raiz): {raiz}\n"));
            for f in &folders { out.push_str(&format!("• {f}: {}\n", notes.iter().filter(|n| &n.folder == f).count())); }
            Ok(out)
        }
        "list_notes" => {
            let tag = gs("tag").map(|t| fold(&format!("#{}", t.trim_start_matches('#'))));
            let l: Vec<String> = notes.iter().filter(|n| in_folder(n, gs("folder"))).filter(|n| tag.as_ref().map_or(true, |t| fold(&n.body).contains(t.as_str()) || n.tags.iter().any(|x| fold(&format!("#{x}")) == *t))).take(gi("limit", 50)).map(meta).collect();
            Ok(if l.is_empty() { "Nenhuma nota encontrada.".into() } else { l.join("\n") })
        }
        "search_notes" => {
            let q = gs("query").unwrap_or("").trim(); if q.is_empty() { return Err("Informe o que buscar".into()); }
            let words: Vec<String> = fold(q).split_whitespace().map(String::from).collect();
            let mut hits: Vec<(usize, &Note, String)> = vec![];
            for n in notes.iter().filter(|n| in_folder(n, gs("folder"))) {
                let (t, b) = (fold(&n.title), if locked(n) { String::new() } else { fold(&n.body) });
                if !words.iter().all(|w| t.contains(w.as_str()) || b.contains(w.as_str())) { continue; }
                let score = words.iter().map(|w| if t.contains(w.as_str()) { 10 } else { 0 } + b.matches(w.as_str()).count()).sum();
                let body: Vec<char> = n.body.chars().collect(); let bf: Vec<char> = b.chars().collect();
                let w0: Vec<char> = words[0].chars().collect();
                let pos = (0..bf.len().saturating_sub(w0.len().saturating_sub(1))).find(|&i| bf[i..].starts_with(&w0)).unwrap_or(0);
                let (s0, s1) = (pos.saturating_sub(70), (pos + 130).min(body.len()));
                let snip: String = body.get(s0..s1).map(|c| c.iter().collect::<String>()).unwrap_or_default().replace('\n', " ");
                hits.push((score, n, snip));
            }
            hits.sort_by(|a, b| b.0.cmp(&a.0));
            if hits.is_empty() { return Ok(format!("Nada encontrado para “{q}”.")); }
            Ok(hits.iter().take(gi("limit", 20)).map(|(_, n, s)| format!("{}\n   …{}…", meta(n), s.trim())).collect::<Vec<_>>().join("\n"))
        }
        "read_note" => Ok(full(pick(&notes, a)?)),
        "create_note" => {
            let title = gs("title").unwrap_or("").trim(); if title.is_empty() { return Err("Informe o título".into()); }
            let folder = gs("folder").unwrap_or("").trim().trim_matches('/').to_string();
            let t = now_iso();
            let n = Note { id: new_id(), title: title.into(), folder: folder.clone(), created: t.clone(), updated: t, body: format!("{}\n", gs("text").unwrap_or("").trim_end()), ..Default::default() };
            save(&root, &n)?;
            Ok(format!("Nota criada: “{}” em {} (id {}). Ela aparece no Marginália em alguns segundos.", n.title, if folder.is_empty() { "(raiz)" } else { folder.as_str() }, n.id))
        }
        "append_to_note" | "replace_in_note" | "update_note" => {
            let mut n = pick(&notes, a)?.clone();
            if locked(&n) { return Err("Essa nota é protegida com senha; não dá para alterar por aqui.".into()); }
            match name {
                "append_to_note" => { let t = gs("text").unwrap_or(""); n.body = format!("{}\n\n{}\n", n.body.trim_end(), t.trim_end()); }
                "replace_in_note" => {
                    let (o, nw) = (gs("old").unwrap_or(""), gs("new").unwrap_or(""));
                    if o.is_empty() { return Err("Informe o trecho antigo".into()); }
                    let c = n.body.matches(o).count();
                    if c == 0 { return Err("O trecho não foi encontrado na nota (leia a nota e copie o trecho exato)".into()); }
                    if c > 1 { return Err(format!("O trecho aparece {c} vezes; mande um trecho maior e único")); }
                    n.body = n.body.replacen(o, nw, 1);
                }
                _ => {
                    if let Some(t) = gs("text") { n.body = format!("{}\n", t.trim_end()); }
                    if let Some(t) = gs("new_title").map(str::trim).filter(|s| !s.is_empty()) { n.title = t.into(); }
                    if let Some(f) = gs("folder") { n.folder = f.trim().trim_matches('/').into(); }
                }
            }
            n.updated = now_iso();
            save(&root, &n)?;
            Ok(format!("Nota “{}” atualizada.", n.title))
        }
        "list_tasks" => {
            let done = a.get("include_done").and_then(|v| v.as_bool()).unwrap_or(false);
            let mut out = vec![];
            for n in notes.iter().filter(|n| in_folder(n, gs("folder")) && !locked(n)) {
                let ts: Vec<String> = n.body.lines().filter(|l| { let t = l.trim_start(); t.starts_with("- [ ]") || t.starts_with("* [ ]") || (done && (t.starts_with("- [x]") || t.starts_with("- [X]"))) }).map(|l| format!("   {}", l.trim())).collect();
                let todo = n.props.get("todo").map(|t| t != "x").unwrap_or(false);
                if ts.is_empty() && !todo { continue; }
                out.push(format!("{}{}", meta(n), if todo { " — (a nota é uma tarefa pendente)" } else { "" }));
                out.extend(ts);
            }
            Ok(if out.is_empty() { "Nenhuma tarefa pendente.".into() } else { out.join("\n") })
        }
        _ => Err(format!("Ferramenta desconhecida: {name}")),
    }
}

pub fn serve() {
    let stdin = std::io::stdin();
    let mut out = std::io::stdout();
    let send = |out: &mut std::io::Stdout, v: Value| { let _ = writeln!(out, "{}", v); let _ = out.flush(); };
    for line in stdin.lock().lines() {
        let Ok(line) = line else { break };
        if line.trim().is_empty() { continue; }
        let Ok(msg) = serde_json::from_str::<Value>(&line) else {
            send(&mut out, json!({"jsonrpc":"2.0","id":null,"error":{"code":-32700,"message":"JSON inválido"}})); continue;
        };
        let id = msg.get("id").cloned();
        let method = msg.get("method").and_then(|m| m.as_str()).unwrap_or("");
        let Some(id) = id else { continue }; // notificações não têm resposta
        let res = match method {
            "initialize" => {
                let pv = msg.pointer("/params/protocolVersion").and_then(|v| v.as_str()).unwrap_or("2025-06-18").to_string();
                Ok(json!({"protocolVersion":pv,"capabilities":{"tools":{"listChanged":false}},
                    "serverInfo":{"name":"marginalia-notes","title":"Marginália Notes","version":env!("CARGO_PKG_VERSION")},
                    "instructions":"Notas do usuário no Marginália Notes (Markdown). Use search_notes/list_notes para achar, read_note para ler e create_note/append_to_note/replace_in_note/update_note para escrever. Escreva em português, salvo pedido contrário. Notas com senha não podem ser lidas."}))
            }
            "ping" => Ok(json!({})),
            "tools/list" => Ok(json!({"tools": tools()})),
            "tools/call" => {
                let name = msg.pointer("/params/name").and_then(|v| v.as_str()).unwrap_or("");
                let args = msg.pointer("/params/arguments").cloned().unwrap_or(json!({}));
                Ok(match call(name, &args) {
                    Ok(t) => json!({"content":[{"type":"text","text":t}],"isError":false}),
                    Err(e) => json!({"content":[{"type":"text","text":e}],"isError":true}),
                })
            }
            "resources/list" => Ok(json!({"resources":[]})),
            "prompts/list" => Ok(json!({"prompts":[]})),
            _ => Err(json!({"code":-32601,"message":format!("Método não suportado: {method}")})),
        };
        match res {
            Ok(r) => send(&mut out, json!({"jsonrpc":"2.0","id":id,"result":r})),
            Err(e) => send(&mut out, json!({"jsonrpc":"2.0","id":id,"error":e})),
        }
    }
}

/// grava o conector na configuração do app Claude para computador
pub fn install_claude_desktop(cmd: &str) -> Result<String, String> {
    let dir = if cfg!(windows) { std::env::var_os("APPDATA").map(|a| PathBuf::from(a).join("Claude")) }
        else if cfg!(target_os = "macos") { home().map(|h| h.join("Library/Application Support/Claude")) }
        else { config_dir().map(|c| c.join("Claude")) };
    let dir = dir.ok_or("Não achei a pasta do Claude")?;
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let f = dir.join("claude_desktop_config.json");
    let mut j: Value = fs::read_to_string(&f).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or(json!({}));
    if !j.is_object() { j = json!({}); }
    if f.exists() { let _ = fs::copy(&f, dir.join("claude_desktop_config.antes-marginalia.json")); }
    if !j.get("mcpServers").map_or(false, |v| v.is_object()) { j["mcpServers"] = json!({}); }
    j["mcpServers"]["marginalia-notes"] = json!({"command": cmd, "args": ["--mcp"]});
    fs::write(&f, serde_json::to_string_pretty(&j).unwrap()).map_err(|e| e.to_string())?;
    Ok(f.to_string_lossy().into())
}
