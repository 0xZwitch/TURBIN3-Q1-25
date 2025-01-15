import { Commitment, Connection, Keypair, PublicKey } from "@solana/web3.js"
import wallet from "../wba-wallet.json"
import { getOrCreateAssociatedTokenAccount, transfer } from "@solana/spl-token";

// We're going to import our keypair from the wallet file
const keypair = Keypair.fromSecretKey(new Uint8Array(wallet));

//Create a Solana devnet connection
const commitment: Commitment = "confirmed";
const connection = new Connection("https://api.devnet.solana.com", commitment);

// Mint address
const mint = new PublicKey("5Z3yjpascdAvJhwd3of5BfBJsU6EZhN7XDRaRMwX4jyS");

// Recipient address
const to = new PublicKey("FnLyawZLBmFHTTdvsi9Zr16FLot7ue5rwUAEJaVo7rNR");

(async () => {
    try {
        // Get the token account of the fromWallet address, and if it does not exist, create it
        const fromTokenAccount = await getOrCreateAssociatedTokenAccount(connection, keypair, mint, keypair.publicKey);

        // Get the token account of the toWallet address, and if it does not exist, create it
        const toTokenAccount = await getOrCreateAssociatedTokenAccount(connection, keypair, mint, to)

        // Transfer the new token to the "toTokenAccount" we just created
        const txId = await transfer(connection, keypair, fromTokenAccount.address, toTokenAccount.address, keypair.publicKey, 1000000)
        console.log('txId: ', txId)
    } catch (e) {
        console.error(`Oops, something went wrong: ${e}`)
    }
})();