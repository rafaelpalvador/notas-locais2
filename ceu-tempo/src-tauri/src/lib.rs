use serde::Serialize;

// ---------- pedidos HTTP feitos pelo Rust (sem limites de CORS do WebView) ----------
#[derive(Serialize)]
struct HttpOut {
    status: u16,
    body: String,
}

#[tauri::command]
async fn http_get(url: String) -> Result<HttpOut, String> {
    if !url.starts_with("https://") {
        return Err("Só são permitidos endereços https".into());
    }
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .user_agent("Ceu-Tempo/0.1 (app pessoal de previsao do tempo)")
        .build()
        .map_err(|e| e.to_string())?;
    let rs = client
        .get(&url)
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|e| e.to_string())?;
    let status = rs.status().as_u16();
    let body = rs.text().await.map_err(|e| e.to_string())?;
    Ok(HttpOut { status, body })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|_app| {
            #[cfg(mobile)]
            _app.handle().plugin(tauri_plugin_geolocation::init())?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![http_get])
        .run(tauri::generate_context!())
        .expect("erro ao iniciar o app");
}
