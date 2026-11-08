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
    { Ok(app.path().document_dir().map_err(|e| e.to_string())?.join("NotasLocais")) }
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .invoke_handler(tauri::generate_handler![load_vault, save_note, delete_note, create_folder, save_asset, read_asset, set_vault, rename_folder, delete_folder, http_req, ext_open, ext_poll, ext_reveal, update_install])
        .run(tauri::generate_context!())
        .expect("erro ao iniciar o app");
}
