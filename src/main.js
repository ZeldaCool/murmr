const { invoke } = window.__TAURI__.core;

window.connect = async function() {
  const ip = document.getElementById("ip").value;
  const status = document.getElementById("status");

  status.innerText = "Connecting to peer...";

  try {
    await invoke("connect", { ip });
    status.innerText = "Connected to peer";
  } catch (err) {
    status.innerText = "Error trying to connect!";
  }

}
