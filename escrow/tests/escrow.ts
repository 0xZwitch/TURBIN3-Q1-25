import * as anchor from "@coral-xyz/anchor";
import { BN, Program } from "@coral-xyz/anchor";
import { PublicKey, SystemProgram } from "@solana/web3.js";
import { createAssociatedTokenAccountIdempotentInstruction, createInitializeMint2Instruction, createMintToInstruction, getAssociatedTokenAddressSync, getMinimumBalanceForRentExemptMint, MINT_SIZE, TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { randomBytes } from "crypto";
import { assert } from "chai";
import { Escrow } from "../target/types/escrow";

describe("escrow", () => {
  anchor.setProvider(anchor.AnchorProvider.env());
  const provider = anchor.getProvider();

  const program = anchor.workspace.Escrow as Program<Escrow>;
  const tokenProgram = TOKEN_PROGRAM_ID;

  const alice = anchor.web3.Keypair.generate();
  const bob = anchor.web3.Keypair.generate();
  const mintA = anchor.web3.Keypair.generate();
  const mintB = anchor.web3.Keypair.generate();

  const seed = new BN(randomBytes(8));
  const depositAmount = new BN(100_000_000)
  const receiveAmount = new BN(120_000_000)
  const tokenAmount = 1_000_000_000

  const aliceMintAPublicKey = getAssociatedTokenAddressSync(mintA.publicKey, alice.publicKey, false, tokenProgram)
  const bobMintAPublicKey = getAssociatedTokenAddressSync(mintA.publicKey, bob.publicKey, false, tokenProgram);
  const bobMintBPublicKey = getAssociatedTokenAddressSync(mintB.publicKey, bob.publicKey, false, tokenProgram);
  const [escrowPublicKey] = PublicKey.findProgramAddressSync(
    [Buffer.from("escrow"), alice.publicKey.toBuffer(), seed.toArrayLike(Buffer, "le", 8)],
    program.programId
  )
  const vaultPublicKey = getAssociatedTokenAddressSync(mintA.publicKey, escrowPublicKey, true, tokenProgram);

  const accounts = {
    maker: alice.publicKey,
    taker: bob.publicKey,
    mintA: mintA.publicKey,
    mintB: mintB.publicKey,
    makerMintAAta: aliceMintAPublicKey,
    takerMintABta: bobMintBPublicKey,
    escrow: escrowPublicKey,
    vault: vaultPublicKey,
    tokenProgram,
  }

  it('should create mints', async () => {
    let lamports = await getMinimumBalanceForRentExemptMint(program.provider.connection);
    await airdrop(provider.connection, alice.publicKey);
    await airdrop(provider.connection, bob.publicKey);

    const transaction = new anchor.web3.Transaction();
    transaction.instructions = [
      SystemProgram.createAccount({
        fromPubkey: provider.publicKey,
        newAccountPubkey: mintA.publicKey,
        lamports,
        space: MINT_SIZE,
        programId: tokenProgram,
      }),
      SystemProgram.createAccount({
        fromPubkey: provider.publicKey,
        newAccountPubkey: mintB.publicKey,
        lamports,
        space: MINT_SIZE,
        programId: tokenProgram,
      }),

      createInitializeMint2Instruction(mintA.publicKey, 6, alice.publicKey, null, tokenProgram),
      createAssociatedTokenAccountIdempotentInstruction(provider.publicKey, aliceMintAPublicKey, alice.publicKey, mintA.publicKey, tokenProgram),
      createMintToInstruction(mintA.publicKey, aliceMintAPublicKey, alice.publicKey, tokenAmount, undefined, tokenProgram),

      createInitializeMint2Instruction(mintB.publicKey, 6, bob.publicKey, null, tokenProgram),
      createAssociatedTokenAccountIdempotentInstruction(provider.publicKey, bobMintBPublicKey, bob.publicKey, mintB.publicKey, tokenProgram),
      createMintToInstruction(mintB.publicKey, bobMintBPublicKey, bob.publicKey, tokenAmount, undefined, tokenProgram),
    ];

    console.log({
      maker: alice.publicKey.toString(),
      taker: bob.publicKey.toString(),
      mintA: mintA.publicKey.toString(),
      mintB: mintB.publicKey.toString(),
      makerMintAAta: aliceMintAPublicKey.toString(),
      takerMintBAta: bobMintBPublicKey.toString()
    });
    await provider.sendAndConfirm(transaction, [alice, bob, mintA, mintB]);
    const connection = program.provider.connection;

    const mintAInfo = await connection.getAccountInfo(mintA.publicKey);
    // console.log("MintA Account Info:", mintAInfo);

    const mintBInfo = await connection.getAccountInfo(mintB.publicKey);
    // console.log("MintB Account Info:", mintBInfo);

    const aliceMintABalance = await connection.getTokenAccountBalance(aliceMintAPublicKey);
    // console.log("Alice Mint A Balance:", aliceMintABalance);

    assert.strictEqual(aliceMintABalance.value.amount, tokenAmount.toString())
  })

  it('should let alice deposit to the escrow\'s vault', async () => {
    const signature = await program.methods.make(seed, receiveAmount, depositAmount).accounts({ ...accounts }).signers([alice]).rpc()
    await confirmTransaction(provider.connection, signature)

    const vaultBalance = await provider.connection.getTokenAccountBalance(vaultPublicKey);
    // console.log("Vault Balance:", vaultBalance);

    assert.strictEqual(vaultBalance.value.amount, depositAmount.toString())
  })

  it('should send bob deposit from vault after purchasing', async () => {
    const signature = await program.methods.take().accountsPartial({ ...accounts }).signers([bob]).rpc()
    await confirmTransaction(provider.connection, signature)

    const bobMintABalance = await provider.connection.getTokenAccountBalance(bobMintAPublicKey);
    // console.log("Bob Mint A Balance :", bobMintABalance);

    assert.strictEqual(bobMintABalance.value.amount, depositAmount.toString())
  })
});

async function airdrop(connection: any, address: any, amount = 1_000_000_000) {
  await connection.confirmTransaction(await connection.requestAirdrop(address, amount), "confirmed");
}

async function confirmTransaction(connection: any, signature: string) {
  const block = await connection.getLatestBlockhash();

  await connection.confirmTransaction({
    signature,
    ...block,
  })

  return signature
}
