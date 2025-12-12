use std::sync::{LazyLock, OnceLock};
use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};
use tauri_plugin_opener::OpenerExt;
use tokio::sync::{oneshot, watch, OnceCell};

#[derive(Copy, Clone, Deserialize, Serialize)]
pub struct RequestedAutofillFields {
    #[serde(default)]
    name: bool,
    #[serde(default)]
    email: bool,
    #[serde(default)]
    phone_number: bool,
    #[serde(default)]
    id: bool,
    #[serde(default)]
    profile_picture: bool,
    #[serde(default)]
    license: bool,
    #[serde(default)]
    id_image: bool,
}

#[derive(Clone, Default, Deserialize, Serialize)]
pub struct AutofillData {
    name: Option<[String; 2]>,
    email: Option<String>,
    phone_number: Option<String>,
    id: Option<String>,
    /// stored as a base64 image
    profile_picture: Option<String>,
    /// stored as a base64 image
    license: Option<String>,
    id_image: Option<String>,
}

static BASE_URL: LazyLock<Url> = LazyLock::new(|| {
    Url::parse("https://absher-zt.vrtgs.xyz/requests/").unwrap()
});

static CLIENT: OnceLock<Client> = OnceLock::new();

fn request_url(code: &str) -> Result<Url, url::ParseError> {
    (*BASE_URL).clone().join(code)
}

fn get_client() -> reqwest::Result<&'static Client> {
    if let Some(val) = CLIENT.get() {
        return Ok(val);
    }

    let client = Client::builder().build()?;
    Ok(CLIENT.get_or_init(|| client))
}

const USER_DATA_PATH: &str = "./user-data.json";

static USER_DATA_CHANNEL: OnceCell<(watch::Sender<AutofillData>, watch::Receiver<AutofillData>)> = OnceCell::new();

async fn get_user_data_channel() -> &'static (watch::Sender<AutofillData>, watch::Receiver<AutofillData>) {
    USER_DATA_CHANNEL
        .get()
        .expect("failed to initalize at the start of teh program")
}

async fn load_data() -> watch::Ref<AutofillData> {
    let (_rx, tx) = get_user_data_channel().await;
    tx.borrow()
}

async fn store_data(user_data: AutofillData) {
    let (rx, _tx) = get_user_data_channel().await;
    rx.send(user_data).expect("user data channel somehow got dropped")
}

#[tauri::command]
pub async fn fetch_request_info(code: &str) -> anyhow::Result<RequestedAutofillFields> {
    let result = get_client()?.get(request_url(code)?)
        .await?
        .json()?;

    Ok(result)
}

#[tauri::command]
pub async fn confirm_request(code: &str) -> anyhow::Result<()> {
    let ref_lock = load_data();
    let request = get_client()?
        .post(request_url(code)?)
        .json(&ref_lock);

    drop(ref_lock);

    request
        .send()
        .await?
        .error_for_status()?;

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let (send_channel, rcv_channel) = oneshot::channel();
    let (started_init, wait_for_start) = oneshot::channel();

    tauri::async_runtime::spawn(USER_DATA_CHANNEL.get_or_init(async || {
        started_init.send(()).unwrap();
        rcv_channel.await.failed_to_initialize()
    }));
    tauri::async_runtime::block_on(wait_for_start).unwrap();

    tauri::async_runtime::spawn(async move {
        let user_data = tokio::fs::read_to_string(USER_DATA_PATH)
            .await
            .and_then(|str| serde_json::from_str(&str))
            .unwrap_or_default();

        let (tx, rx) = watch::channel(user_data);

        send_channel.send((tx, rx))
    });

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
