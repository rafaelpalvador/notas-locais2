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
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![load_vault, save_note, delete_note, create_folder, save_asset, read_asset, set_vault, rename_folder, delete_folder])
        .run(tauri::generate_context!())
        .expect("erro ao iniciar o app");
}
