import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { Vault } from "../target/types/vault";
import { expect } from "chai";
import { getAssociatedTokenAddress} from "@solana/spl-token";

describe("vault", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.Vault as Program<Vault>;
  const vaultAdmin = anchor.web3.Keypair.generate();
  const mint = anchor.web3.Keypair.generate();
  const signer = anchor.getProvider().publicKey;

  it("should initialize program", async () => {
    await program.methods.initialize().accounts({
        vaultAdmin: vaultAdmin.publicKey,
    }).signers([vaultAdmin]).rpc();
    const adminAccount = await program.account.vaultAdmin.fetch(vaultAdmin.publicKey);
    const admin = adminAccount.admin.toBase58();
    expect(admin).eq(signer.toBase58());
  });

  it("should initialize mint", async () => {
    await program.methods.initializeMint().accounts({
        vaultAdmin: vaultAdmin.publicKey,
        mint: mint.publicKey,
    }).signers([mint]).rpc();
  });

  it("should initialize account", async () => {
    const tokenAccount = await getAssociatedTokenAddress(mint.publicKey, signer);
    await program.methods.initializeAccount().accounts({
        vaultAdmin: vaultAdmin.publicKey,
        mint: mint.publicKey,
        tokenAccount: tokenAccount
    }).rpc();
  });

  it("should mint tokens", async() => {
    const tokenAccount = await getAssociatedTokenAddress(mint.publicKey, signer);
    await program.methods.mintToken(new anchor.BN(123)).accounts({
        vaultAdmin: vaultAdmin.publicKey,
        mint: mint.publicKey,
        tokenAccount: tokenAccount,
    }).rpc();
    const signerBalance = await anchor.getProvider().connection.getTokenAccountBalance(tokenAccount);
    expect(signerBalance.value.amount).eq("123");
  });

});
