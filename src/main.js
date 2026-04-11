const { invoke } = window.__TAURI__.core;

let isConnected = false;

window.connect = async function() {
  const ip = document.getElementById("ip").value;

  try {
    await invoke("connect", { ip });
    console.log("Connecting!1!!!!11");
    isConnected = true;
    render();
  } catch (err) {
    console.log("Error :(");
  }

}

function render() {
  document.getElementById("conscreen").style.display =
    isConnected ? "none" : "block";
  document.getElementById("callscrn").style.display = 
    isConnected ? "block" : "none";
}

window.mute = async function() {
  try {
    await invoke("toggle_mic");
    console.log("Mic toggled");
  } catch (err) {
    console.log("Error :(");
  }
}

window.volup = async function() {
  try {
    console.log("Volume up");
  } catch (err) {
    console.log("Error :(");
  }
}

window.voldown = async function() {
  try {
    console.log("Volume down");
  } catch (err) {
    console.log("Error :(");
  }  
}
