import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { Vault } from "../target/types/vault";
import { expect } from "chai";
import { getAssociatedTokenAddress, transfer } from "@solana/spl-token";

describe("vault", () => {
    // Configure the client to use the local cluster.
    anchor.setProvider(anchor.AnchorProvider.env());

    const program = anchor.workspace.Vault as Program<Vault>;
    const vaultAdmin = anchor.web3.Keypair.generate();
    const mint = anchor.web3.Keypair.generate();
    const signer = anchor.getProvider().publicKey;
    const depositCell = anchor.web3.Keypair.generate();
    const depositor = anchor.web3.Keypair.generate();

    const connection = anchor.getProvider().connection;

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
            mint: mint.publicKey,
            tokenAccount: tokenAccount
        }).rpc();
    });

    it("should mint tokens", async () => {
        const tokenAccount = await getAssociatedTokenAddress(mint.publicKey, signer);
        await program.methods.mintToken(new anchor.BN(123)).accounts({
            vaultAdmin: vaultAdmin.publicKey,
            mint: mint.publicKey,
            tokenAccount: tokenAccount,
        }).rpc();
        const signerBalance = await connection.getTokenAccountBalance(tokenAccount);
        expect(signerBalance.value.amount).eq("123");
    });

    it("should provide some native currency to depositor", async () => {
        const transaction = new anchor.web3.Transaction().add(anchor.web3.SystemProgram.transfer({
            fromPubkey: signer,
            toPubkey: depositor.publicKey,
            lamports: anchor.web3.LAMPORTS_PER_SOL * 3,
        }));
        await anchor.getProvider().sendAndConfirm(transaction);
    });

    it("should initialize depositor's account", async () => {
        const tokenAccount = await getAssociatedTokenAddress(mint.publicKey, depositor.publicKey);
        await program.methods.initializeAccount().accounts({
            tokenAccount,
            mint: mint.publicKey,
            authority: depositor.publicKey,
        }).signers([depositor]).rpc();
    });

    it("should mint tokens to depositor", async () => {
        const tokenAccount = await getAssociatedTokenAddress(mint.publicKey, depositor.publicKey);
        await program.methods.mintToken(new anchor.BN(123)).accounts({
            authority: signer,
            mint: mint.publicKey,
            vaultAdmin: vaultAdmin.publicKey,
            tokenAccount
        }).rpc();
    });

    it("should deposit tokens", async () => {
        const userAccount = await getAssociatedTokenAddress(mint.publicKey, depositor.publicKey);
        const vaultAccount = await getAssociatedTokenAddress(mint.publicKey, signer);
        await program.methods.depositTokens(new anchor.BN(10)).accounts({
            depositCell: depositCell.publicKey,
            mint: mint.publicKey,
            authority: depositor.publicKey,
            userAccount,
            vaultAccount,
        }).signers([depositor, depositCell]).rpc();
    });

});
