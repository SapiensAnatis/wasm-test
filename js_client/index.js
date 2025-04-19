import * as signalR from '@microsoft/signalr'

const connection = new signalR.HubConnectionBuilder()
  .withUrl("http://localhost:5095/chatHub")
  .configureLogging(signalR.LogLevel.Debug)
  .build();

async function start() {
  try {
    await connection.start();
    console.log("SignalR connected.");
  } catch (err) {
    console.error(err);
    setTimeout(start, 5000);
  }
}

connection.onclose(async () => {
  await start();
});

connection.on("ReceiveMessage", (user, message) => {
  console.log(`${user}: ${message}`);
});

async function helloWorld() {
  try {
    await connection.invoke("SendMessage", "index.js", "Hello World!");
  } catch (err) {
    console.error(err);
  }
}

start();

setInterval(helloWorld, 1000);
