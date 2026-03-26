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
const BOB = process.env.BOB;

const abi = JSON.parse(fs.readFileSync(CONTRACT_ABI, "utf8"));
const contract = new ContractPromise(api, abi, CONTRACT);
const keyring = new Keyring({ type: "sr25519" });
const bob = keyring.addFromUri(BOB);

const gasLimit = api.registry.createType('WeightV2', {
          refTime: 300000000000,
          proofSize: 500000,
});
const storageDepositLimit = null;

await new Promise(async (resolve, reject) => {
  const unsub = await contract.tx
    .open({ storageDepositLimit, gasLimit }, 
    )
    .signAndSend(bob, ({ status, events, data }) => {
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