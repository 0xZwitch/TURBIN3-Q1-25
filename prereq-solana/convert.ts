import bs58 from "bs58";
import promptSync from "prompt-sync";

const prompt = promptSync();

function base58ToWallet() {
  const base58Key = prompt("Enter your base58 key:");
  const wallet = bs58.decode(base58Key);
  console.log('Wallet: ', wallet)
}

function walletToBase58() {
  const wallet = prompt("Enter your wallet file:");
  const base58Key = bs58.encode(Buffer.from(JSON.parse(wallet)));
  console.log('Base58 Key: ', base58Key)
}

base58ToWallet()
// walletToBase58()
