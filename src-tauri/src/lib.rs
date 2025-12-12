use std::sync::{LazyLock, Once, OnceLock};
use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};
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

#[derive(Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
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

#[derive(Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
pub struct AutofillDataRef<'a> {
    name: Option<[&'a str; 2]>,
    email: Option<&'a str>,
    phone_number: Option<&'a str>,
    id: Option<&'a str>,
    /// stored as a base64 image
    profile_picture: Option<&'a str>,
    /// stored as a base64 image
    license: Option<&'a str>,
    id_image: Option<&'a str>,
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

static USER_DATA_CHANNEL: OnceCell<(watch::Sender<AutofillData>, watch::Receiver<AutofillData>)> = OnceCell::const_new();

async fn get_user_data_channel() -> &'static (watch::Sender<AutofillData>, watch::Receiver<AutofillData>) {
    USER_DATA_CHANNEL
        .get()
        .expect("failed to initalize at the start of teh program")
}

async fn with_load_data<T>(fun: impl FnOnce(&AutofillData) -> T) -> T {
    let (_rx, tx) = get_user_data_channel().await;
    fun(&tx.borrow())
}

async fn store_data(user_data: AutofillData) {
    let (rx, _tx) = get_user_data_channel().await;
    rx.send(user_data).expect("user data channel somehow got dropped")
}

#[tauri::command]
async fn load_data_from_store() -> AutofillData {
    with_load_data(AutofillData::clone).await
}

#[tauri::command]
async fn store_data_to_store(data: AutofillData) {
    eprintln!("submitted data store request!");
    store_data(data).await;
}

#[tauri::command]
async fn fetch_request_info(code: &str) -> Result<RequestedAutofillFields, String> {
    async fn fetch_request_info(code: &str) -> anyhow::Result<RequestedAutofillFields> {
        let result = get_client()?
            .get(request_url(code)?)
            .send()
            .await?
            .json()
            .await?;

        Ok(result)
    }

    fetch_request_info(code).await.map_err(|err| err.to_string())
}

#[tauri::command]
async fn confirm_request(code: &str, filter: RequestedAutofillFields) -> Result<(), String> {
    async fn confirm_request(code: &str, filter: RequestedAutofillFields) -> anyhow::Result<()> {
        let client = get_client()?;
        let url = request_url(code)?;
        let request = with_load_data(|autofill_data| {
            let name = filter
                .name
                .then(|| {
                    autofill_data
                        .name
                        .as_ref()
                        .map(|names| names.each_ref().map(String::as_str))
                })
                .flatten();

            let json = AutofillDataRef {
                name,
                email: filter.email.then_some(autofill_data.email.as_deref()).flatten(),
                phone_number: filter.phone_number.then_some(autofill_data.phone_number.as_deref()).flatten(),
                id: filter.id.then_some(autofill_data.id.as_deref()).flatten(),
                profile_picture: filter.profile_picture.then_some(autofill_data.profile_picture.as_deref()).flatten(),
                license: filter.license.then_some(autofill_data.license.as_deref()).flatten(),
                id_image: filter.id_image.then_some(autofill_data.id_image.as_deref()).flatten(),
            };

            client.post(url).json(&json)
        }).await;

        request
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    confirm_request(code, filter).await.map_err(|err| err.to_string())
}


fn setup_once(
    _app: &mut tauri::App,
    wait_for_start: oneshot::Receiver<()>,
    send_channel: oneshot::Sender<(watch::Sender<AutofillData>, watch::Receiver<AutofillData>)>
) {
    #[cfg(target_os = "android")]
    {
        use tauri::Manager;

        let app = _app;
        std::env::set_current_dir(app.path().app_data_dir().unwrap())
            .unwrap();
    }

    #[cfg(target_vendor = "apple")]
    {
        std::env::set_current_dir(
            "/Users/vrtgs/Coding/Rust/android-apps/absher-zt/dist"
        ).unwrap();
    }


    tauri::async_runtime::block_on(wait_for_start).unwrap();

    tauri::async_runtime::spawn(async move {
        let user_data = tokio::fs::read_to_string(USER_DATA_PATH)
            .await
            .map_err(anyhow::Error::new)
            .and_then(|str| serde_json::from_str(&str).map_err(anyhow::Error::new))
            .unwrap_or_else(|err| {
                eprintln!("failed to get user information: {err}");
                AutofillData {
                    name: Some(["foo", "bar"].map(str::to_string)),
                    email: Some("mail@mail.com".to_string()),
                    phone_number: Some("0567814779".to_owned()),
                    id: Some("1122334466".to_owned()),
                    ..AutofillData::default()
                }
            });

        let (tx, mut rx) = watch::channel::<AutofillData>(user_data.clone());
        send_channel.send((tx, rx.clone()))
            .ok()
            .expect("failed to send");

        let mut current_user_data = user_data;

        let mut get_data = async move |current_data: &mut AutofillData| -> bool {
            rx
                .wait_for(|data| *data != *current_data)
                .await
                .map(|ref_data| {
                    *current_data = (*ref_data).clone();
                })
                .is_ok()
        };

        eprintln!("waiting for data store request!");

        while get_data(&mut current_user_data).await {
            eprintln!("updated!!!");
            let json = serde_json::to_string(&current_user_data).unwrap();
            tauri::async_runtime::spawn_blocking(move || {
                std::fs::write(USER_DATA_PATH, json)
            }).await.unwrap().unwrap();
        }
    });
}


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {

    let (send_channel, rcv_channel) =
        oneshot::channel::<(watch::Sender<AutofillData>, watch::Receiver<AutofillData>)>();
    let (started_init, wait_for_start) = oneshot::channel();

    tauri::async_runtime::spawn(USER_DATA_CHANNEL.get_or_init(async move || {
        started_init.send(()).unwrap();
        rcv_channel.await.expect("failed to initialize user data DB")
    }));

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            fetch_request_info,
            confirm_request,
            load_data_from_store,
            store_data_to_store
        ])
        .setup(|app| {
            static ONCE: Once = Once::new();
            ONCE.call_once(|| setup_once(app, wait_for_start, send_channel));
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
