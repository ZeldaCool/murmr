const { invoke } = window.__TAURI__.core;

let isConnected = false;
let lanIp;
let Ip;

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

window.getiplan = async function() {
    lanIp = document.querySelector("#ipp");
    try {
      lanIp.textContent = await invoke("getlanip");
      let ip = await invoke("getlanip");
      console.log("Ip: " + ip);
    } catch (err) {
      console.log("Error: " + err);
    }
}

window.getip = async function() {
  Ip = document.querySelector("#ippp");
  try {
    Ip.textContent = await invoke("getip");
    let ip = await invoke("getip");
    console.log("Ip: " + ip);
  } catch (err) {
    console.log("Error: " + err);
  }
}

window.getiplantwo = async function() {
  lanIp = document.querySelector("#ippp");
    try {
      lanIp.textContent = await invoke("getlanip");
    } catch (err) {
      console.log("Error: " + err);
    }

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
