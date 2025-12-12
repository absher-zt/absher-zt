import { invoke } from "@tauri-apps/api/core";

type RequestedData = {
  name: boolean;
  email: boolean;
  phone_number: boolean;
  id: boolean;
  profile_picture: boolean;
  license: boolean;
  id_image: boolean;
}

async function fetch_request_info(code: string): Promise<RequestedData> {
  return (await invoke("requested_data", { code })) as RequestedData;
}


async function confirm_request(code: string): Promise<void> {
  (await invoke("confirm_request", { code }))
}

// @ts-ignore
let _UNUSED = [fetch_request_info, confirm_request];
