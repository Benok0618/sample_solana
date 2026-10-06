const express = require("express");
const http = require("http");
const WebSocket = require("ws");
const anchor = require("@coral-xyz/anchor");
const { Connection, Keypair, PublicKey } = require("@solana/web3.js");
const fs = require("fs");
const os = require("os");
const path = require("path");

const idl = require("./idl/hello_solana.json");
const PROGRAM_ID = new PublicKey(idl.address);

const DEVICE_IDS = ["temp-1", "humidity-1", "pressure-1"];

const app = express();
app.use(express.static(path.join(__dirname, "public")));

const server = http.createServer(app);
const wss = new WebSocket.Server({ server });

function broadcast(data) {
  const msg = JSON.stringify(data);
  wss.clients.forEach((client) => {
    if (client.readyState === WebSocket.OPEN) client.send(msg);
  });
}

let programInstance = null;

async function getProgram() {
  if (programInstance) return programInstance;
  const connection = new Connection("http://127.0.0.1:8899", "confirmed");
  const walletPath = path.join(os.homedir(), ".config/solana/id.json");
  const secretKey = Uint8Array.from(JSON.parse(fs.readFileSync(walletPath, "utf8")));
  const wallet = Keypair.fromSecretKey(secretKey);
  const provider = new anchor.AnchorProvider(connection, new anchor.Wallet(wallet), {
    commitment: "confirmed",
  });
  programInstance = new anchor.Program(idl, provider);
  return programInstance;
}

app.get("/api/devices", (req, res) => {
  res.json(DEVICE_IDS);
});

app.get("/api/readings", async (req, res) => {
  try {
    const deviceId = req.query.device || DEVICE_IDS[0];
    const program = await getProgram();

    const [devicePda] = PublicKey.findProgramAddressSync(
      [Buffer.from("device"), Buffer.from(deviceId)],
      PROGRAM_ID
    );

    const device = await program.account.device.fetch(devicePda);
    const allReadings = await program.account.reading.all();

    const readings = allReadings
      .filter((r) => r.account.device.equals(devicePda))
      .map((r) => ({
        index: r.account.index.toString(),
        value: r.account.value.toString(),
        timestamp: r.account.timestamp.toString(),
      }))
      .sort((a, b) => Number(a.index) - Number(b.index));

    res.json({
      deviceId,
      active: device.active,
      readingCount: device.readingCount.toString(),
      lastValue: device.lastValue.toString(),
      readings: readings.slice(-50),
    });
  } catch (err) {
    res.status(500).json({ error: err.message || String(err) });
  }
});

async function watchEvents() {
  const program = await getProgram();
  console.log("Listening for ThresholdExceeded events...");

  program.addEventListener("thresholdExceeded", (event) => {
    console.log("⚠ ThresholdExceeded:", event.deviceId, event.value.toString());
    broadcast({
      type: "threshold_exceeded",
      deviceId: event.deviceId,
      value: event.value.toString(),
      timestamp: event.timestamp.toString(),
    });
  });
}

const PORT = 3000;
server.listen(PORT, () => {
  console.log(`Dashboard API + WebSocket running at http://localhost:${PORT}`);
  watchEvents().catch((err) => console.error("Event listener failed to start:", err));
});