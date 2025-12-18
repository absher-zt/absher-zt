use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};
use serde_wasm_bindgen as swb;
use anyhow::Result;
use leptos::{view, IntoView};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

fn display_to_anyhow(err: impl ToString) -> anyhow::Error {
    anyhow::Error::msg(err.to_string())
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct RequestedDataMask {
    pub name: bool,
    pub email: bool,
    pub phone_number: bool,
    pub id: bool,
    pub profile_picture: bool,
    pub license: bool,
    pub id_image: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestedData {
    pub name: [String; 2],
    pub email: String,
    pub phone_number: String,
    pub id: String,
    pub profile_picture: String,
    pub license: String,
    pub id_image: String,
}

pub async fn fetch_request_info(code: &str) -> Result<RequestedDataMask> {
    #[derive(Serialize)]
    struct FetchRequestInfo<'a> {
        code: &'a str
    }
    let args = swb::to_value(&FetchRequestInfo {
        code
    }).map_err(display_to_anyhow)?;

    let out = invoke("fetch_request_info", args).await;

    swb::from_value(out).map_err(display_to_anyhow)
}

pub async fn load_data_from_store() -> Result<RequestedData> {
    let args = JsValue::from(js_sys::Object::new());
    let out = invoke("load_data_from_store", args).await;
    swb::from_value(out).map_err(display_to_anyhow)
}

pub async fn store_data_to_store(data: &RequestedData) -> Result<()> {
    #[derive(Serialize)]
    struct StoreDataRequest<'a> {
        data: &'a RequestedData
    }
    let args = swb::to_value(&StoreDataRequest {
        data
    }).map_err(display_to_anyhow)?;

    let _ = invoke("store_data_to_store", args).await;
    Ok(())
}

pub async fn confirm_request(code: &str, filter: &RequestedDataMask) -> Result<()> {
    #[derive(Serialize)]
    struct ConfirmRequest<'a> {
        code: &'a str,
        filter: &'a RequestedDataMask
    }

    let args = swb::to_value(&ConfirmRequest {
        code,
        filter
    }).map_err(display_to_anyhow)?;

    let _ = invoke("confirm_request", args).await;
    Ok(())
}

#[leptos::component]
pub fn App() -> impl IntoView {
    view! {
        <div></div>
    }
}