use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};
use serde_wasm_bindgen as swb;
use anyhow::Result;
use leptos::{component, view, IntoView};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
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
    pub name: Vec<String>,
    pub email: String,
    pub phone_number: String,
    pub id: String,
    pub profile_picture: String,
    pub license: String,
    pub id_image: String,
}

pub async fn fetch_request_info(code: &str) -> Result<RequestedDataMask> {
    #[derive(Serialize, Deserialize)]
    struct FetchRequestInfo<'a> {
        code: &'a str
    }
    let args = swb::to_value(&FetchRequestInfo {
        code
    })?;

    let out = invoke("fetch_request_info", args).await;

    swb::from_value(out).map_err(anyhow::Error::new)
}

pub async fn load_data_from_store() -> Result<RequestedData> {
    let args = JsValue::from(js_sys::Object::new());
    let out = invoke("load_data_from_store", args).await;
    swb::from_value(out).map_err(anyhow::Error::new)
}

pub async fn store_data_to_store(data: &RequestedData) -> Result<()> {
    #[derive(Serialize, Deserialize)]
    struct StoreDataRequest<'a> {
        data: &'a RequestedData
    }
    let args = swb::to_value(&StoreDataRequest {
        data
    })?;

    let _ = invoke("store_data_to_store", args).await;
    Ok(())
}

pub async fn confirm_request(code: &str, filter: &RequestedDataMask) -> Result<()> {
    #[derive(Serialize, Deserialize)]
    struct ConfirmRequest<'a> {
        code: &'a str,
        filter: &'a RequestedDataMask
    }

    let args = swb::to_value(&ConfirmRequest {
        code,
        filter
    })?;

    let _ = invoke("confirm_request", args).await;
    Ok(())
}


#[component]
pub fn App() -> impl IntoView {
    view! {
        <div></div>
    }
}