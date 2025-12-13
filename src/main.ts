import {invoke} from "@tauri-apps/api/core";

type RequestedDataMask = {
  name: boolean;
  email: boolean;
  phone_number: boolean;
  id: boolean;
  profile_picture: boolean;
  license: boolean;
  id_image: boolean;
}

type RequestedData = {
  name: string[];
  email: string;
  phone_number: string;
  id: string;
  profile_picture: string;
  license: string;
  id_image: string;
}

async function fetch_request_info(code: string): Promise<RequestedDataMask> {
  return (await invoke("fetch_request_info", { code })) as RequestedDataMask;
}

async function load_data_from_store(): Promise<RequestedData> {
  return (await invoke("load_data_from_store", {})) as RequestedData;
}

async function store_data_to_store(data: RequestedData): Promise<void> {
  await invoke("store_data_to_store", { data })
}


async function confirm_request(code: string, filter: RequestedDataMask): Promise<void> {
  (await invoke("confirm_request", { code, filter }))
}

const autofill = get("entercode");
const confirmer = get("confirm");
const devmode = get("devmode");
const throbber = get("throbber");

const codein = get("codein") as HTMLInputElement;
const accesser = get("accesser");
const nodevmode = get("nodevmode");
const canceller = get("canceller");

let inside = false;
function get(id: string) {
  let autofill = document.getElementById(id);
  if (autofill == null)
  {
    switch_to(null);
    throw new Error("oops");
  }
  return autofill;
}

function getq(query: string) {
  let autofill = document.querySelector(query);
  if (autofill == null)
  {
    switch_to(null);
    throw new Error("oops");
  }
  return autofill;
}

function getImage(element: HTMLInputElement): Promise<string> {
  return new Promise((res, rej) => {
    if (element.files == null || element.files[0] == null) {
      res("data:image/png;base64,");
      return;
    }
    let file = element.files[0];
    let r = new FileReader();
    r.addEventListener("load", () => {
      res(r.result as string);
    });
    r.addEventListener("error", e => {
      rej(e);
    });
    r.readAsDataURL(file);
  });
}

function switch_to(window: HTMLElement | null = null) {
  autofill.style.display = "none";
  confirmer.style.display = "none";
  devmode.style.display = "none";
  throbber.style.display = "none";
  if (window != null)
    window.style.display = "block";

  inside = true;
  setTimeout(() => {
    inside = window != null;
  }, 500);
}

function main() {
  let key = "";

  canceller.addEventListener("click", () => {
    switch_to(null);
  });
  document.body.addEventListener("click", async e => {
    let bottomness= e.y > window.innerHeight * 2/3;
    if (inside)
      return;
    if (bottomness) {
      let info = await load_data_from_store();
      (getq("input[data-absher=name]") as HTMLInputElement).value = info.name[0] + " " + info.name[1];
      (getq("input[data-absher=email]") as HTMLInputElement).value = info.email;
      (getq("input[data-absher=phone_number]") as HTMLInputElement).value = info.phone_number;
      (getq("input[data-absher=id]") as HTMLInputElement).value = info.id;
      switch_to(devmode);
    } else {
      codein.value = "";
      switch_to(autofill);
    }
  });
  codein.addEventListener("input", async () => {
    key = codein.value;
    key = key.toUpperCase();
    key = key.replace(/[^A-Z]/g, "");
    codein.value = key;
    if (key.length == 9) {
      switch_to(throbber);
      let data;
      try {
        data = await fetch_request_info(key);
      } catch (e) {
        document.body.innerHTML = "";
        document.body.innerText = (e as any).toString();
        return;
      }
      for (const [key, val] of Object.entries(data)) {
        let element = document.querySelector(`tr[data-absher="${key}"]`);
        if (element == null)
        {
          switch_to(null);
          return;
        }
        (element as HTMLTableRowElement).style.display = val ? "block" : "none";
      }
      for (let element of document.querySelectorAll("tr input[type=checkbox]"))
        (element as HTMLInputElement).checked = true;
      switch_to(confirmer);
    }
  });

  accesser.addEventListener("click", async e => {
    switch_to(throbber);
    e.preventDefault();
    let dict: RequestedDataMask = {
      name: false,
      email: false,
      phone_number: false,
      id: false,
      profile_picture: false,
      license: false,
      id_image: false,
    };
    for (const key of Object.keys(dict)) {
      let element = document.querySelector(`tr[data-absher="${key}"] input`);
      if (element == null)
      {
        switch_to(null);
        return;
      }
      // @ts-ignore
      dict[key] = (element as HTMLInputElement).checked;
    }

    await confirm_request(key, dict);
    switch_to(null);
    return 0;
  });

  nodevmode.addEventListener("click", async () => {
    switch_to(null);
    try {
      let names = (getq("input[data-absher=name]") as HTMLInputElement).value.split(" ");

      let [profile_picture, license, id_image] = await Promise.all([
        getImage(getq("input[data-absher=profile_picture]") as HTMLInputElement),
        getImage(getq("input[data-absher=license]") as HTMLInputElement),
        getImage(getq("input[data-absher=id_image]") as HTMLInputElement),
      ]);

      let first_name = names[0] ?? "";
      let last_name = names.slice(1).join(' ');
      let info: RequestedData = {
        name: [first_name, last_name],
        email: (getq("input[data-absher=email]") as HTMLInputElement).value,
        phone_number: (getq("input[data-absher=phone_number]") as HTMLInputElement).value,
        id: (getq("input[data-absher=id]") as HTMLInputElement).value,
        profile_picture,
        license,
        id_image
      };

      await store_data_to_store(info);
    } catch (e) {
      document.body.innerHTML = "";
      document.body.innerText = (e as any).toString();
      return;
    }
  });
}

main();