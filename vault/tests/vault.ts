import * as anchor from "@coral-xyz/anchor";
import { BN, Program } from "@coral-xyz/anchor";
import { PublicKey } from "@solana/web3.js";
import { Vault } from "../target/types/vault";
import { assert } from "chai";

describe("vault", () => {
  anchor.setProvider(anchor.AnchorProvider.env());

  const provider = anchor.getProvider();

  const program = anchor.workspace.Vault as Program<Vault>;

  const alice = anchor.web3.Keypair.generate();
  const aliceInitialBalance = 10_000_000;
  const depositAmount = 2_000_000;
  const withdrawAmount = 1_000_000;

  const vaultState = {
    vault_state: "state",
    vault: "vault",
  };

  const [vault_state_pkey, vault_state_bump] = getAddress(vaultState.vault_state, alice.publicKey, program.programId);
  const [vault_pkey, vault_bump] = getAddress(vaultState.vault, vault_state_pkey, program.programId);

  describe('Initialize', () => {
    it('should initialize vault state', async () => {
      await airdrop(provider.connection, alice.publicKey, aliceInitialBalance);

      await program.methods.initialize().accounts({
        signer: alice.publicKey,
      }).signers([alice]).rpc()

      let vaultStateData = await program.account.vaultState.fetch(vault_state_pkey);

      if (vaultStateData) {
        assert.strictEqual(vaultStateData.vaultStateBump.toString(), vault_state_bump.toString())
        assert.strictEqual(vaultStateData.vaultBump.toString(), vault_bump.toString())
      }
    })
  })

  describe('Payment', () => {
    it('should transfer certain amount of lamport from signer to vault', async () => {
      const signature = await program.methods.deposit(new BN(depositAmount)).accounts({
        signer: alice.publicKey,
      }).signers([alice]).rpc()

      await confirm(provider.connection, signature)

      const vaultBalance = await program.provider.connection.getBalance(vault_pkey)

      assert.strictEqual(vaultBalance.toString(), depositAmount.toString())
    })

    it('should transfer certain amount of lamport from vault to signer', async () => {
      const signature = await program.methods.withdraw(new BN(withdrawAmount)).accounts({
        signer: alice.publicKey,
      }).signers([alice]).rpc()

      await confirm(provider.connection, signature)

      const vaultBalance = await program.provider.connection.getBalance(vault_pkey)

      const vaultNewBalance = depositAmount - withdrawAmount

      assert.strictEqual(vaultBalance.toString(), vaultNewBalance.toString())
    })
  })

  describe('Close', () => {
    it('should emptied and close vault state', async () => {
      await program.methods.close().accounts({ signer: alice.publicKey }).signers([alice]).rpc()

      const vaultBalance = await program.provider.connection.getBalance(vault_pkey)

      assert.strictEqual(vaultBalance.toString(), "0")

      let vaultStateAccount = await program.account.vaultState.fetchNullable(vault_state_pkey);

      assert.strictEqual(vaultStateAccount, null)
    })
  })
});

async function airdrop(connection: any, address: any, amount = 1000000000) {
  await connection.confirmTransaction(await connection.requestAirdrop(address, amount), "confirmed");
}

function getAddress(seed: string, signer: PublicKey, programID: PublicKey) {
  return PublicKey.findProgramAddressSync(
    [
      anchor.utils.bytes.utf8.encode(seed),
      signer.toBuffer()
    ], programID);
}

async function confirm(connection: any, signature: string) {
  const latestBlock = await connection.getLatestBlockhash()
  await connection.confirmTransaction({
    signature,
    ...latestBlock,
  })
}
