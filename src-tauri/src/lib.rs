use serde::{Deserialize, Serialize};
use std::{fs, path::{Path, PathBuf}};
use tauri::Manager;

#[derive(Serialize, Deserialize, Clone, Default)]
struct Note {
    id: String, title: String, folder: String, created: String, updated: String, body: String,
    #[serde(default)] tags: Vec<String>,
    #[serde(default)] props: std::collections::BTreeMap<String, String>,
}
#[derive(Serialize)]
struct Vault { notes: Vec<Note>, folders: Vec<String>, root: String }

// Cofre = ~/Documents/NotasLocais (pastas reais + arquivos .md + assets/)
fn default_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    #[cfg(mobile)]
    { Ok(app.path().app_data_dir().map_err(|e| e.to_string())?.join("NotasLocais")) }
    #[cfg(not(mobile))]
    {
        // sem pasta Documentos configurada (ex.: Linux mínimo): usa ~/Documents, senão a pasta de dados do app
        let base = app.path().document_dir().ok()
            .or_else(|| app.path().home_dir().ok().map(|h| h.join("Documents")))
            .or_else(|| app.path().app_data_dir().ok())
            .ok_or_else(|| "Não achei uma pasta para guardar as notas".to_string())?;
        Ok(base.join("NotasLocais"))
    }
}
fn cfg(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let d = app.path().app_config_dir().map_err(|e| e.to_string())?;
    fs::create_dir_all(&d).map_err(|e| e.to_string())?;
    Ok(d.join("vault.txt"))
}
fn root(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let custom = cfg(app).ok().and_then(|p| fs::read_to_string(p).ok()).map(|s| s.trim().to_string()).filter(|s| !s.is_empty());
    let r = match custom {
        Some(p) => PathBuf::from(p),
        None => default_dir(app)?,
    };
    fs::create_dir_all(r.join("assets")).map_err(|e| e.to_string())?;
    Ok(r)
}
#[tauri::command]
fn set_vault(app: tauri::AppHandle, path: String) -> Result<String, String> {
    let c = cfg(&app)?;
    let p = path.trim();
    if p.is_empty() {
        let _ = fs::remove_file(&c);
    } else {
        fs::create_dir_all(Path::new(p).join("assets")).map_err(|e| e.to_string())?;
        fs::write(&c, p).map_err(|e| e.to_string())?;
    }
    Ok(root(&app)?.to_string_lossy().into())
}
fn safe(s: &str) -> String {
    let t: String = s.chars().map(|c| if "\\/:*?\"<>|".contains(c) { '-' } else { c }).collect();
    let t = t.trim().trim_matches('.').to_string();
    if t.is_empty() { "Sem título".into() } else { t }
}
fn parse(text: &str, folder: &str, stem: &str) -> Note {
    let mut n = Note { title: stem.into(), folder: folder.into(), body: text.into(), ..Default::default() };
    if let Some(rest) = text.strip_prefix("---\n") {
        if let Some((fm, body)) = rest.split_once("\n---") {
            for l in fm.lines() {
                if let Some((k, v)) = l.split_once(':') {
                    let v = v.trim();
                    let v = v.strip_prefix('"').unwrap_or(v);
                    let v = v.strip_suffix('"').unwrap_or(v).replace("\\\"", "\"");
                    match k.trim() { "id" => n.id = v, "title" => n.title = v, "created" => n.created = v, "updated" => n.updated = v, "tags" | "folder" => {}, key => { n.props.insert(key.to_string(), v); } }
                }
            }
            n.body = body.trim_start_matches('\n').to_string();
        }
    }
    n
}
fn render(note: &Note) -> String {
    let extra: String = note.props.iter().filter(|(k, _)| !k.is_empty() && !k.contains(':') && !k.contains('\n'))
        .map(|(k, v)| format!("\n{}: \"{}\"", k, v.replace('"', "\\\""))).collect();
    format!(
        "---\nid: \"{}\"\ntitle: \"{}\"\ncreated: {}\nupdated: {}\ntags: [{}]{}\n---\n\n{}",
        note.id, note.title.replace('"', "\\\""), note.created, note.updated, note.tags.join(", "), extra, note.body
    )
}
fn walk(dir: &Path, base: &Path, notes: &mut Vec<Note>, folders: &mut Vec<String>) {
    let Ok(rd) = fs::read_dir(dir) else { return };
    for e in rd.flatten() {
        let p = e.path();
        let rel = p.strip_prefix(base).unwrap();
        if p.is_dir() {
            let name = e.file_name().to_string_lossy().to_string();
            if name == "assets" || name.starts_with('.') { continue; }
            folders.push(rel.to_string_lossy().replace('\\', "/"));
            walk(&p, base, notes, folders);
        } else if p.extension().map_or(false, |x| x == "md") {
            if let Ok(t) = fs::read_to_string(&p) {
                let folder = rel.parent().map(|x| x.to_string_lossy().replace('\\', "/")).unwrap_or_default();
                notes.push(parse(&t, &folder, &p.file_stem().unwrap().to_string_lossy()));
            }
        }
    }
}
fn find(dir: &Path, id: &str) -> Option<PathBuf> {
    for e in fs::read_dir(dir).ok()?.flatten() {
        let p = e.path();
        if p.is_dir() {
            if e.file_name() == "assets" { continue; }
            if let Some(x) = find(&p, id) { return Some(x); }
        } else if p.extension().map_or(false, |x| x == "md") {
            if let Ok(t) = fs::read_to_string(&p) {
                let key = format!("id: \"{}\"", id);
                if t.lines().take(8).any(|l| l.trim() == key) { return Some(p); }
            }
        }
    }
    None
}
#[tauri::command]
fn load_vault(app: tauri::AppHandle) -> Result<Vault, String> {
    let r = root(&app)?;
    let (mut notes, mut folders) = (vec![], vec![]);
    walk(&r, &r, &mut notes, &mut folders);
    folders.sort();
    Ok(Vault { notes, folders, root: r.to_string_lossy().into() })
}
#[tauri::command]
fn save_note(app: tauri::AppHandle, note: Note) -> Result<(), String> {
    let r = root(&app)?;
    let mut dir = r.clone();
    for s in note.folder.split('/').filter(|s| !s.is_empty() && *s != "..") { dir.push(safe(s)); }
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let old = find(&r, &note.id);
    let mut path = dir.join(format!("{}.md", safe(&note.title)));
    if path.exists() && old.as_ref() != Some(&path) {
        path = dir.join(format!("{} ({}).md", safe(&note.title), &note.id[..note.id.len().min(4)]));
    }
    let text = render(&note);
    fs::write(&path, text).map_err(|e| e.to_string())?;
    if let Some(o) = old { if o != path { let _ = fs::remove_file(o); } }
    Ok(())
}
#[tauri::command]
fn delete_note(app: tauri::AppHandle, id: String) -> Result<(), String> {
    let r = root(&app)?;
    if let Some(p) = find(&r, &id) { fs::remove_file(p).map_err(|e| e.to_string())?; }
    Ok(())
}
#[tauri::command]
fn create_folder(app: tauri::AppHandle, path: String) -> Result<(), String> {
    let mut d = root(&app)?;
    for s in path.split('/').filter(|s| !s.is_empty() && *s != "..") { d.push(safe(s)); }
    fs::create_dir_all(d).map_err(|e| e.to_string())
}
#[tauri::command]
fn save_asset(app: tauri::AppHandle, name: String, data: Vec<u8>) -> Result<(), String> {
    fs::write(root(&app)?.join("assets").join(safe(&name)), data).map_err(|e| e.to_string())
}
#[tauri::command]
fn read_asset(app: tauri::AppHandle, name: String) -> Result<Vec<u8>, String> {
    fs::read(root(&app)?.join("assets").join(safe(&name))).map_err(|e| e.to_string())
}
fn safe_path(base: &Path, rel: &str) -> PathBuf {
    let mut p = base.to_path_buf();
    for s in rel.split('/').filter(|s| !s.is_empty() && *s != "..") { p.push(safe(s)); }
    p
}
fn prune_empty(d: &Path) -> bool {
    let Ok(rd) = fs::read_dir(d) else { return true };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() { if !prune_empty(&p) { return false; } } else { return false; }
    }
    fs::remove_dir(d).is_ok()
}
fn do_rename(base: &Path, from: &str, to: &str) -> Result<(), String> {
    let (a, b) = (safe_path(base, from), safe_path(base, to));
    if !a.exists() { return fs::create_dir_all(&b).map_err(|e| e.to_string()); }
    if b.exists() { return Err("A pasta de destino já existe.".into()); }
    if let Some(p) = b.parent() { fs::create_dir_all(p).map_err(|e| e.to_string())?; }
    fs::rename(&a, &b).map_err(|e| e.to_string())
}
#[tauri::command]
fn rename_folder(app: tauri::AppHandle, from: String, to: String) -> Result<(), String> {
    do_rename(&root(&app)?, &from, &to)
}
#[tauri::command]
fn delete_folder(app: tauri::AppHandle, path: String) -> Result<(), String> {
    let r = root(&app)?;
    let p = safe_path(&r, &path);
    if p != r { prune_empty(&p); }
    Ok(())
}
// ---------- HTTP (WebDAV) sem limites de CORS ----------
#[derive(Serialize)]
struct HttpOut { status: u16, body: String, headers: std::collections::BTreeMap<String, String> }
#[tauri::command]
async fn http_req(method: String, url: String, headers: std::collections::BTreeMap<String, String>, body: Option<String>) -> Result<HttpOut, String> {
    use base64::Engine;
    let b64 = base64::engine::general_purpose::STANDARD;
    let m = reqwest::Method::from_bytes(method.as_bytes()).map_err(|e| e.to_string())?;
    let client = reqwest::Client::builder().timeout(std::time::Duration::from_secs(60)).build().map_err(|e| e.to_string())?;
    let mut rq = client.request(m, &url);
    for (k, v) in headers.iter() { rq = rq.header(k.as_str(), v.as_str()); }
    if let Some(b) = body { rq = rq.body(b64.decode(b).map_err(|e| e.to_string())?); }
    let rs = rq.send().await.map_err(|e| e.to_string())?;
    let status = rs.status().as_u16();
    let mut hs = std::collections::BTreeMap::new();
    for (k, v) in rs.headers().iter() { if let Ok(s) = v.to_str() { hs.insert(k.as_str().to_string(), s.to_string()); } }
    let bytes = rs.bytes().await.map_err(|e| e.to_string())?;
    Ok(HttpOut { status, body: b64.encode(&bytes), headers: hs })
}
// ---------- editor externo ----------
#[derive(Serialize)]
struct ExtOut { path: String, mtime: u64, body: String }
fn mtime_ms(p: &Path) -> u64 {
    fs::metadata(p).and_then(|m| m.modified()).ok().and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map(|d| d.as_millis() as u64).unwrap_or(0)
}
fn note_file(app: &tauri::AppHandle, id: &str) -> Result<PathBuf, String> {
    let r = root(app)?;
    find(&r, id).ok_or_else(|| "Nota ainda não salva no disco".to_string())
}
#[tauri::command]
fn ext_open(app: tauri::AppHandle, id: String, with: Option<String>) -> Result<ExtOut, String> {
    let p = note_file(&app, &id)?;
    #[cfg(mobile)]
    { let _ = (&with, &p); return Err("Editor externo não está disponível no celular".into()); }
    #[cfg(not(mobile))]
    {
        let ps = p.to_string_lossy().to_string();
        let w = with.unwrap_or_default();
        let mut cmd: std::process::Command;
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            if w.is_empty() { cmd = std::process::Command::new("cmd"); cmd.args(["/C", "start", "", ps.as_str()]); }
            else { cmd = std::process::Command::new("cmd"); cmd.args(["/C", w.as_str(), ps.as_str()]); }
            cmd.creation_flags(0x08000000);
        }
        #[cfg(target_os = "macos")]
        { cmd = std::process::Command::new("open"); if !w.is_empty() { cmd.args(["-a", w.as_str()]); } cmd.arg(&ps); }
        #[cfg(all(unix, not(target_os = "macos")))]
        { if w.is_empty() { cmd = std::process::Command::new("xdg-open"); cmd.arg(&ps); } else { cmd = std::process::Command::new(&w); cmd.arg(&ps); } }
        cmd.spawn().map_err(|e| e.to_string())?;
        let t = fs::read_to_string(&p).map_err(|e| e.to_string())?.replace("\r\n", "\n");
        let stem = p.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
        let n = parse(&t, "", &stem);
        Ok(ExtOut { path: ps, mtime: mtime_ms(&p), body: n.body })
    }
}

#[tauri::command]
fn ext_reveal(app: tauri::AppHandle, id: String) -> Result<(), String> {
    let p = note_file(&app, &id)?;
    #[cfg(mobile)]
    { let _ = p; return Err("Indisponível no celular".into()); }
    #[cfg(not(mobile))]
    {
        let ps = p.to_string_lossy().to_string();
        #[cfg(target_os = "windows")]
        { use std::os::windows::process::CommandExt; std::process::Command::new("explorer").arg(format!("/select,{}", ps)).creation_flags(0x08000000).spawn().map_err(|e| e.to_string())?; }
        #[cfg(target_os = "macos")]
        { std::process::Command::new("open").args(["-R", ps.as_str()]).spawn().map_err(|e| e.to_string())?; }
        #[cfg(all(unix, not(target_os = "macos")))]
        { let d = p.parent().map(|x| x.to_string_lossy().to_string()).unwrap_or(ps); std::process::Command::new("xdg-open").arg(d).spawn().map_err(|e| e.to_string())?; }
        Ok(())
    }
}

// ---------- atualização automática (Windows): baixa o instalador da versão nova e roda em modo passivo ----------
#[tauri::command]
async fn update_install(url: String, token: Option<String>) -> Result<(), String> {
    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    { let _ = (&url, &token); return Err("A atualização automática pelo app é só para Windows e Linux (AppImage)".into()); }
    #[cfg(any(target_os = "windows", target_os = "linux"))]
    {
        #[cfg(target_os = "linux")]
        let target = match std::env::var_os("APPIMAGE") {
            Some(a) => PathBuf::from(a),
            None => {
                // instalado pelo comando do terminal: o próprio script baixa, troca e reabre o app
                let home = std::env::var_os("HOME").map(PathBuf::from).unwrap_or_default();
                let script = home.join(".local/bin/marginalia-atualizar");
                if !script.exists() { return Err("Para atualizar esta instalação, rode no terminal: marginalia-atualizar".into()); }
                use std::os::unix::process::CommandExt;
                let log = fs::File::create(std::env::temp_dir().join("marginalia-atualizar.log")).map_err(|e| e.to_string())?;
                let log2 = log.try_clone().map_err(|e| e.to_string())?;
                std::process::Command::new("bash").arg(&script).arg("--relancar")
                    .env_remove("APPDIR").env_remove("APPIMAGE").env_remove("LD_LIBRARY_PATH").env_remove("GIO_MODULE_DIR").env_remove("GDK_PIXBUF_MODULE_FILE").env_remove("GST_PLUGIN_SYSTEM_PATH").env_remove("GST_PLUGIN_SYSTEM_PATH_1_0").env_remove("GTK_PATH").env_remove("XDG_DATA_DIRS")
                    .stdin(std::process::Stdio::null()).stdout(log).stderr(log2).process_group(0)
                    .spawn().map_err(|e| e.to_string())?;
                let _ = (&url, &token);
                return Ok(());
            }
        };
        let client = reqwest::Client::builder().timeout(std::time::Duration::from_secs(300)).build().map_err(|e| e.to_string())?;
        let mut rq = client.get(&url).header("User-Agent", "MarginaliaNotes").header("Accept", "application/octet-stream");
        if let Some(t) = token.filter(|t| !t.trim().is_empty()) { rq = rq.header("Authorization", format!("Bearer {}", t.trim())); }
        let rs = rq.send().await.map_err(|e| e.to_string())?;
        if !rs.status().is_success() { return Err(format!("Download falhou ({})", rs.status())); }
        let bytes = rs.bytes().await.map_err(|e| e.to_string())?;
        if bytes.len() < 100_000 { return Err("Arquivo de atualização inválido".into()); }
        #[cfg(target_os = "windows")]
        {
            let path = std::env::temp_dir().join("Marginalia-atualizacao.exe");
            fs::write(&path, &bytes).map_err(|e| e.to_string())?;
            use std::os::windows::process::CommandExt;
            std::process::Command::new(&path).args(["/P", "/R"]).creation_flags(0x00000008).spawn().map_err(|e| e.to_string())?;
        }
        #[cfg(target_os = "linux")]
        {
            use std::os::unix::fs::PermissionsExt;
            if &bytes[..4] != b"\x7fELF" { return Err("Arquivo AppImage inválido".into()); }
            let tmp = target.with_extension("novo");
            fs::write(&tmp, &bytes).map_err(|e| format!("Sem permissão para gravar ao lado do AppImage: {e}"))?;
            fs::set_permissions(&tmp, fs::Permissions::from_mode(0o755)).map_err(|e| e.to_string())?;
            fs::rename(&tmp, &target).map_err(|e| e.to_string())?;
            std::process::Command::new(&target).spawn().map_err(|e| e.to_string())?;
        }
        std::thread::spawn(|| { std::thread::sleep(std::time::Duration::from_millis(700)); std::process::exit(0); });
        Ok(())
    }
}
#[tauri::command]
fn ext_poll(app: tauri::AppHandle, id: String) -> Result<ExtOut, String> {
    let p = note_file(&app, &id)?;
    let t = fs::read_to_string(&p).map_err(|e| e.to_string())?.replace("\r\n", "\n");
    let stem = p.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
    let n = parse(&t, "", &stem);
    Ok(ExtOut { path: p.to_string_lossy().into(), mtime: mtime_ms(&p), body: n.body })
}



// ---------- Login do Google (OAuth com retorno para 127.0.0.1) ----------
mod oauth {
    use std::{sync::Mutex, time::Duration};
    pub static RESULT: Mutex<Option<String>> = Mutex::new(None);
    pub fn listen() -> Result<u16, String> {
        let server = tiny_http::Server::http("127.0.0.1:0").map_err(|e| e.to_string())?;
        let port = server.server_addr().to_ip().map(|a| a.port()).ok_or("sem porta")?;
        if let Ok(mut r) = RESULT.lock() { *r = None; }
        std::thread::spawn(move || {
            let deadline = std::time::Instant::now() + Duration::from_secs(600);
            while std::time::Instant::now() < deadline {
                let rq = match server.recv_timeout(Duration::from_millis(500)) { Ok(Some(r)) => r, Ok(None) => continue, Err(_) => break };
                let url = rq.url().to_string();
                if !url.contains("code=") && !url.contains("error=") { let _ = rq.respond(tiny_http::Response::from_string("").with_status_code(404)); continue; }
                let html = "<!doctype html><meta charset=utf-8><meta name=viewport content='width=device-width'><body style='font-family:system-ui;text-align:center;padding:48px 20px;background:#faf7f0;color:#2b2622'><h2>Pronto! ✔</h2><p>Você entrou com o Google. Pode fechar esta aba e voltar ao <b>Marginália Notes</b>.</p></body>";
                let mut resp = tiny_http::Response::from_string(html);
                if let Ok(h) = tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..]) { resp.add_header(h); }
                let _ = rq.respond(resp);
                if let Ok(mut r) = RESULT.lock() { *r = Some(url); }
                break;
            }
        });
        Ok(port)
    }
}
#[tauri::command]
fn oauth_listen() -> Result<u16, String> { oauth::listen() }
#[tauri::command]
fn oauth_poll() -> Option<String> { oauth::RESULT.lock().ok().and_then(|mut r| r.take()) }

// ---------- Sincronização pelo Wi-Fi: servidor WebDAV mínimo no computador ----------
mod lan {
    use std::{fs, io::Read, path::{Path, PathBuf}, sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}}, time::Duration};
    pub struct Lan { pub stop: Arc<AtomicBool>, pub port: u16 }
    pub static STATE: Mutex<Option<Lan>> = Mutex::new(None);

    fn etag(b: &[u8]) -> String { let mut h: u64 = 0xcbf29ce484222325; for x in b { h ^= *x as u64; h = h.wrapping_mul(0x100000001b3); } format!("\"{:016x}-{}\"", h, b.len()) }
    fn decode(p: &str) -> String {
        let b = p.as_bytes(); let mut o = Vec::with_capacity(b.len()); let mut i = 0;
        while i < b.len() { if b[i] == b'%' && i + 2 < b.len() { if let Ok(v) = u8::from_str_radix(&p[i + 1..i + 3], 16) { o.push(v); i += 3; continue; } } o.push(b[i]); i += 1; }
        String::from_utf8_lossy(&o).into_owned()
    }
    fn safe(root: &Path, url: &str) -> Option<PathBuf> {
        let path = url.split('?').next().unwrap_or("");
        let mut out = root.to_path_buf();
        for part in decode(path).split('/') { if part.is_empty() || part == "." { continue; } if part == ".." || part.contains('\\') || part.contains(':') { return None; } out.push(part); }
        Some(out)
    }
    pub fn local_ips() -> Vec<String> {
        let mut v = Vec::new();
        if let Ok(s) = std::net::UdpSocket::bind("0.0.0.0:0") { if s.connect("192.168.1.1:80").is_ok() || s.connect("8.8.8.8:80").is_ok() { if let Ok(a) = s.local_addr() { let ip = a.ip().to_string(); if ip != "0.0.0.0" { v.push(ip); } } } }
        if let Ok(s) = std::net::UdpSocket::bind("0.0.0.0:0") { if s.connect("100.100.100.100:80").is_ok() { if let Ok(a) = s.local_addr() { let ip = a.ip().to_string(); if ip.starts_with("100.") && !v.contains(&ip) { v.push(ip); } } } }
        v
    }
    pub fn start(root: PathBuf, port: u16, auth: String) -> Result<(), String> {
        let mut st = STATE.lock().map_err(|e| e.to_string())?;
        if let Some(l) = st.take() { l.stop.store(true, Ordering::SeqCst); std::thread::sleep(Duration::from_millis(700)); }
        fs::create_dir_all(&root).map_err(|e| e.to_string())?;
        let server = tiny_http::Server::http(("0.0.0.0", port)).map_err(|e| format!("Não consegui abrir a porta {port}: {e}"))?;
        let stop = Arc::new(AtomicBool::new(false)); let stop2 = stop.clone();
        let want = format!("Basic {}", auth);
        std::thread::spawn(move || {
            while !stop2.load(Ordering::SeqCst) {
                let mut rq = match server.recv_timeout(Duration::from_millis(500)) { Ok(Some(r)) => r, Ok(None) => continue, Err(_) => break };
                let hdr = |rq: &tiny_http::Request, n: &str| rq.headers().iter().find(|h| h.field.as_str().as_str().eq_ignore_ascii_case(n)).map(|h| h.value.as_str().to_string());
                let mk = |code: u16, body: Vec<u8>, tag: Option<String>| { let mut r = tiny_http::Response::from_data(body).with_status_code(code); if let Some(t) = tag { if let Ok(h) = tiny_http::Header::from_bytes(&b"ETag"[..], t.as_bytes()) { r.add_header(h); } } r };
                if hdr(&rq, "Authorization").as_deref() != Some(want.as_str()) {
                    let mut r = mk(401, b"auth".to_vec(), None);
                    if let Ok(h) = tiny_http::Header::from_bytes(&b"WWW-Authenticate"[..], &b"Basic realm=\"Marginalia\""[..]) { r.add_header(h); }
                    let _ = rq.respond(r); continue;
                }
                let p = match safe(&root, rq.url()) { Some(p) => p, None => { let _ = rq.respond(mk(400, vec![], None)); continue; } };
                let method = rq.method().as_str().to_uppercase();
                let resp = match method.as_str() {
                    "GET" => match fs::read(&p) { Ok(b) if p.is_file() => { let t = etag(&b); mk(200, b, Some(t)) } _ => mk(404, vec![], None) },
                    "PUT" => {
                        let mut body = Vec::new();
                        if rq.as_reader().read_to_end(&mut body).is_err() { mk(400, vec![], None) } else {
                            let cur = fs::read(&p).ok().map(|b| etag(&b));
                            match hdr(&rq, "If-Match") {
                                Some(m) if Some(m.trim().to_string()) != cur => mk(412, vec![], cur),
                                _ => {
                                    if let Some(d) = p.parent() { let _ = fs::create_dir_all(d); }
                                    let tmp = p.with_extension("tmp-up");
                                    match fs::write(&tmp, &body).and_then(|_| fs::rename(&tmp, &p)) { Ok(_) => { let t = etag(&body); mk(if cur.is_some() { 204 } else { 201 }, vec![], Some(t)) } Err(_) => mk(500, vec![], None) }
                                }
                            }
                        }
                    }
                    "DELETE" => { if p.is_file() { let _ = fs::remove_file(&p); mk(204, vec![], None) } else { mk(404, vec![], None) } }
                    "MKCOL" => { if p.is_dir() { mk(405, vec![], None) } else if fs::create_dir_all(&p).is_ok() { mk(201, vec![], None) } else { mk(500, vec![], None) } }
                    _ => mk(200, vec![], None),
                };
                let _ = rq.respond(resp);
            }
        });
        *st = Some(Lan { stop, port });
        Ok(())
    }
    pub fn stop() { if let Ok(mut st) = STATE.lock() { if let Some(l) = st.take() { l.stop.store(true, Ordering::SeqCst); } } }
}

#[derive(Serialize)]
struct LanInfo { running: bool, port: u16, ips: Vec<String> }

#[tauri::command]
fn lan_start(app: tauri::AppHandle, port: u16, user: String, pass: String) -> Result<LanInfo, String> {
    use base64::Engine;
    let root = app.path().app_data_dir().map_err(|e| e.to_string())?.join("sync-wifi");
    let auth = base64::engine::general_purpose::STANDARD.encode(format!("{user}:{pass}"));
    lan::start(root, port, auth)?;
    Ok(LanInfo { running: true, port, ips: lan::local_ips() })
}
#[tauri::command]
fn lan_stop() -> Result<(), String> { lan::stop(); Ok(()) }
#[tauri::command]
fn lan_info() -> LanInfo {
    let port = lan::STATE.lock().ok().and_then(|s| s.as_ref().map(|l| l.port));
    LanInfo { running: port.is_some(), port: port.unwrap_or(0), ips: lan::local_ips() }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![load_vault, save_note, delete_note, create_folder, save_asset, read_asset, set_vault, rename_folder, delete_folder, http_req, ext_open, ext_poll, ext_reveal, update_install, lan_start, lan_stop, lan_info, oauth_listen, oauth_poll])
        .run(tauri::generate_context!())
        .expect("erro ao iniciar o app");
}
