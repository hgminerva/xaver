import { ApiPromise, WsProvider } from "@polkadot/api";
import 'dotenv/config';

const RPC = process.env.RPC;

console.log("Connecting to blockchain...");
const rpc_endpoint = new WsProvider(RPC);
const api = await ApiPromise.create({ provider: rpc_endpoint });
console.log("Connected to:", (await api.rpc.system.chain()).toHuman());

process.exit(0);