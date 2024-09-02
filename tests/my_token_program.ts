import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { MyTokenProgram } from "../target/types/my_token_program";
import {
  TOKEN_PROGRAM_ID,
  MINT_SIZE,
  createAssociatedTokenAccountInstruction,
  getAssociatedTokenAddress,
  createInitializeMintInstruction,
} from "@solana/spl-token";
import { assert } from "chai";

const getProvider = () => anchor.AnchorProvider.env();
const createTransaction = () => new anchor.web3.Transaction();

describe("my-token-program", () => {
  const provider = getProvider();
  anchor.setProvider(provider);

  const program = anchor.workspace.MyTokenProgram as Program<MyTokenProgram>;
  const mintKey: anchor.web3.Keypair = anchor.web3.Keypair.generate();
  let associatedTokenAccount: anchor.web3.PublicKey | undefined;

  it("Mint a token", async () => {
    try {
      const walletPublicKey = provider.wallet.publicKey;
      const lamports = await provider.connection.getMinimumBalanceForRentExemption(MINT_SIZE);
      associatedTokenAccount = await getAssociatedTokenAddress(mintKey.publicKey, walletPublicKey);

      console.log("Creating mint account with address:", mintKey.publicKey.toString());
      console.log("Creating associated token account:", associatedTokenAccount.toString());

      const mintTransaction = createTransaction().add(
        anchor.web3.SystemProgram.createAccount({
          fromPubkey: walletPublicKey,
          newAccountPubkey: mintKey.publicKey,
          space: MINT_SIZE,
          programId: TOKEN_PROGRAM_ID,
          lamports,
        }),
        createInitializeMintInstruction(mintKey.publicKey, 0, walletPublicKey, walletPublicKey),
        createAssociatedTokenAccountInstruction(walletPublicKey, associatedTokenAccount, walletPublicKey, mintKey.publicKey)
      );

      await provider.sendAndConfirm(mintTransaction, [mintKey]);

      const mintInfo = await program.provider.connection.getParsedAccountInfo(mintKey.publicKey);
      console.log("Mint info:", mintInfo);

      console.log(`Minting 10 tokens to: ${associatedTokenAccount.toString()}`);
      await program.methods.mintToken().accounts({
        mint: mintKey.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        tokenAccount: associatedTokenAccount,
        authority: walletPublicKey,
      }).rpc();

      const minted = (await program.provider.connection.getParsedAccountInfo(associatedTokenAccount)).value.data.parsed.info.tokenAmount.amount;
      console.log(`Minted amount in associated token account: ${minted}`);
      assert.equal(minted, 10);

    } catch (error) {
      console.error("Error during minting:", error);
    }
  });

  it("Transfer token", async () => {
    try {
      const walletPublicKey = provider.wallet.publicKey;
      const toWallet = anchor.web3.Keypair.generate();
      const toATA = await getAssociatedTokenAddress(mintKey.publicKey, toWallet.publicKey);

      console.log("Creating associated token account for transfer to:", toWallet.publicKey.toString());
      console.log("Associated token account:", toATA.toString());

      const transferTransaction = createTransaction().add(
        createAssociatedTokenAccountInstruction(walletPublicKey, toATA, toWallet.publicKey, mintKey.publicKey)
      );

      await provider.sendAndConfirm(transferTransaction, []);

      console.log(`Transferring 5 tokens from ${associatedTokenAccount.toString()} to ${toATA.toString()}`);
      await program.methods.transferToken().accounts({
        tokenProgram: TOKEN_PROGRAM_ID,
        from: associatedTokenAccount,
        fromAuthority: walletPublicKey,
        to: toATA,
      }).rpc();

      const fromBalance = (await program.provider.connection.getParsedAccountInfo(associatedTokenAccount)).value.data.parsed.info.tokenAmount.amount;
      const toBalance = (await program.provider.connection.getParsedAccountInfo(toATA)).value.data.parsed.info.tokenAmount.amount;

      console.log(`Balance of from account after transfer: ${fromBalance}`);
      console.log(`Balance of to account after transfer: ${toBalance}`);
      assert.equal(fromBalance, 5);
      assert.equal(toBalance, 5);

    } catch (error) {
      console.error("Error during token transfer:", error);
    }
  });


});
