import 'dotenv/config';
import fs from 'fs';

import { ApiPromise, WsProvider } from "@polkadot/api";
import { ContractPromise } from "@polkadot/api-contract";
import { Keyring } from "@polkadot/keyring";

import { decode } from "./decode.js";

const RPC = process.env.RPC;

console.log("Connecting to blockchain...");
const rpc_endpoint = new WsProvider(RPC);
const api = await ApiPromise.create({ provider: rpc_endpoint });
console.log("Connected to:", (await api.rpc.system.chain()).toHuman());

const CONTRACT = process.env.CONTRACT;
const CONTRACT_ABI = process.env.CONTRACT_ABI;
const ALICE = process.env.ALICE;

const abi = JSON.parse(fs.readFileSync(CONTRACT_ABI, "utf8"));
const contract = new ContractPromise(api, abi, CONTRACT);
const keyring = new Keyring({ type: "sr25519" });
const alice = keyring.addFromUri(ALICE);

const gasLimit = api.registry.createType('WeightV2', {
          refTime: 300000000000,
          proofSize: 500000,
});
const storageDepositLimit = null;

const assetId = 1984; // USDT
const stableAssetId = 1000000003; // XAV
const operator = "XqDGJ69MXL1WhHZiQHsA8HJTu7auK3ZePQZJetMrq3GT5smso"; // Bob
const price = 100;
const share = 1;
const maximumStakes = 1000;
const duration = 1000; // 1,000 Blocks

await new Promise(async (resolve, reject) => {
  const unsub = await contract.tx
    .setup({ storageDepositLimit, gasLimit }, 
        assetId,
        stableAssetId,
        operator,
        price,
        share,
        maximumStakes,
        duration,
    )
    .signAndSend(alice, ({ status, events, data }) => {
      console.log("Status:", status?.type);
      if(events?.length > 0) {
        events.forEach(({ event }) => {
          if (event.section === "contracts" && event.method === "ContractEmitted") {
            console.log(decode(event.data));
            unsub();
            resolve();
          }
        });
      }
    });
});

process.exit(0);