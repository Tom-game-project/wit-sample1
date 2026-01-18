import { invoke } from "@tauri-apps/api/core";

let greetInputEl: HTMLInputElement | null;
let greetMsgEl: HTMLElement | null;

const itemInput = document.querySelector<HTMLInputElement>("#item-input");
const itemList = document.querySelector<HTMLUListElement>("#item-list");
const addItemBtn = document.querySelector<HTMLButtonElement>("#add-item");

async function greet() {
  if (greetMsgEl && greetInputEl) {
    greetMsgEl.textContent = await invoke("greet", {
      name: greetInputEl.value,
    });
  }
}

// =====================
// DB 操作
// =====================
async function refreshItems() {
  const items: { id: number; value: string }[] =
    await invoke("list_items");

  if (!itemList) return;
  itemList.innerHTML = "";

  for (const item of items) {
    const li = document.createElement("li");
    li.textContent = item.value;

    const btn = document.createElement("button");
    btn.textContent = "delete";
    btn.onclick = async () => {
      await invoke("delete_item", { id: item.id });
      refreshItems();
    };

    li.appendChild(btn);
    itemList.appendChild(li);
  }
}

window.addEventListener("DOMContentLoaded", () => {
  greetInputEl = document.querySelector("#greet-input");
  greetMsgEl = document.querySelector("#greet-msg");

  document.querySelector("#greet-form")?.addEventListener("submit", (e) => {
    e.preventDefault();
    greet();
  });

  addItemBtn?.addEventListener("click", async () => {
    if (!itemInput?.value) return;
    await invoke("add_item", { value: itemInput.value });
    itemInput.value = "";
    refreshItems();
  });

  refreshItems();
});

