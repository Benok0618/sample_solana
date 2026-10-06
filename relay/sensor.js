const anchor = require("@coral-xyz/anchor");
const { Connection, Keypair, PublicKey, LAMPORTS_PER_SOL } = require("@solana/web3.js");
const fs = require("fs");
const os = require("os");
const path = require("path");

const idl = require("./idl/hello_solana.json");
const PROGRAM_ID = new PublicKey(idl.address);

const DEVICES = [
  { id: "temp-1", unit: "°C", normal: [15, 35], spike: [1000, 1500], spikeChance: 0.1, interval: 3000 },
  { id: "humidity-1", unit: "%", normal: [30, 70], spike: [1000, 1400], spikeChance: 0.08, interval: 4000 },
  { id: "pressure-1", unit: "hPa", normal: [980, 1030], spike: [1050, 1600], spikeChance: 0.05, interval: 5000 },
];

async function getProgram() {
  const connection = new Connection("http://127.0.0.1:8899", "confirmed");
  const walletPath = path.join(os.homedir(), ".config/solana/id.json");
  const secretKey = Uint8Array.from(JSON.parse(fs.readFileSync(walletPath, "utf8")));
  const wallet = Keypair.fromSecretKey(secretKey);

  const balance = await connection.getBalance(wallet.publicKey);
  if (balance < LAMPORTS_PER_SOL) {
    console.log("Airdropping 2 SOL...");
    const sig = await connection.requestAirdrop(wallet.publicKey, 2 * LAMPORTS_PER_SOL);
    await connection.confirmTransaction(sig);
  }

  const provider = new anchor.AnchorProvider(connection, new anchor.Wallet(wallet), {
    commitment: "confirmed",
  });
  anchor.setProvider(provider);
  return { program: new anchor.Program(idl, provider), wallet };
}

async function ensureRegistered(program, wallet, deviceId) {
  const [devicePda] = PublicKey.findProgramAddressSync(
    [Buffer.from("device"), Buffer.from(deviceId)],
    PROGRAM_ID
  );

  try {
    const account = await program.account.device.fetch(devicePda);
    console.log(`[${deviceId}] already registered, reading_count=${account.readingCount}`);
    return devicePda;
  } catch {
    console.log(`[${deviceId}] registering...`);
    await program.methods
      .registerDevice(deviceId)
      .accounts({
        admin: wallet.publicKey,
        deviceAuthority: wallet.publicKey,
        device: devicePda,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();
    console.log(`[${deviceId}] registered.`);
    return devicePda;
  }
}

function randomIn([min, max]) {
  return Math.floor(min + Math.random() * (max - min));
}

async function startDeviceLoop(program, wallet, config, devicePda) {
  let device = await program.account.device.fetch(devicePda);

  setInterval(async () => {
    const readingCount = device.readingCount;
    const [readingPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("reading"), devicePda.toBuffer(), readingCount.toArrayLike(Buffer, "le", 8)],
      PROGRAM_ID
    );

    const spike = Math.random() < config.spikeChance;
    const value = spike ? randomIn(config.spike) : randomIn(config.normal);
    const timestamp = Math.floor(Date.now() / 1000);

    try {
      const sig = await program.methods
        .submitReading(config.id, new anchor.BN(value), new anchor.BN(timestamp))
        .accounts({
          payer: wallet.publicKey,
          deviceAuthority: wallet.publicKey,
          device: devicePda,
          reading: readingPda,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();

      console.log(
        `[${new Date().toLocaleTimeString()}] ${config.id} #${readingCount} = ${value}${config.unit}${spike ? "  ⚠ SPIKE" : ""}  (${sig.slice(0, 8)}...)`
      );

      device = await program.account.device.fetch(devicePda);
    } catch (err) {
      console.error(`${config.id} submit failed:`, err.message || err);
    }
  }, config.interval);
}

async function main() {
  const { program, wallet } = await getProgram();
  console.log("Using wallet:", wallet.publicKey.toBase58());
  console.log(`Starting ${DEVICES.length} simulated device(s)...\n`);

  for (const config of DEVICES) {
    const devicePda = await ensureRegistered(program, wallet, config.id);
    startDeviceLoop(program, wallet, config, devicePda);
  }
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});