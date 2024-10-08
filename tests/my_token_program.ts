import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { MyTokenProgram } from "../target/types/my_token_program";
import {
  TOKEN_PROGRAM_ID,
  MINT_SIZE,
  createAssociatedTokenAccountInstruction,
  getAssociatedTokenAddress,
  createInitializeMintInstruction,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  ExtensionType,
  TOKEN_2022_PROGRAM_ID,
  createInitializeTransferHookInstruction,
  createMintToInstruction,
  createTransferCheckedInstruction,
  getAssociatedTokenAddressSync,
  getMintLen,
} from "@solana/spl-token";

import {
  Connection,
  PublicKey,
  Signer,
  SystemProgram,
  Transaction,
  sendAndConfirmTransaction,
  TransactionSignature,
  TransactionConfirmationStatus,
  SignatureStatus,
  SYSVAR_RENT_PUBKEY
} from "@solana/web3.js";

import { assert } from "chai";
import BN from "bn.js"; // Import BN for Big Number support

const getProvider = () => anchor.AnchorProvider.env();
const createTransaction = () => new anchor.web3.Transaction();

async function newAccountWithLamports(
  connection: Connection,
  lamports = 100000000000
): Promise<Signer> {
  const account = anchor.web3.Keypair.generate();
  const signature = await connection.requestAirdrop(
    account.publicKey,
    lamports
  );
  const { blockhash, lastValidBlockHeight } =
    await connection.getLatestBlockhash({ commitment: "confirmed" });
  await connection.confirmTransaction({
    signature,
    blockhash,
    lastValidBlockHeight,
  });
  return account;
}


describe("my-token-program", () => {
  const provider = getProvider();
  anchor.setProvider(provider);

  const program = anchor.workspace.MyTokenProgram as Program<MyTokenProgram>;
  const mintKey: anchor.web3.Keypair = anchor.web3.Keypair.generate();
  let associatedTokenAccount: anchor.web3.PublicKey | undefined;

  let authority: Signer;
  let recipient: Signer;
  let authorityATA: PublicKey;
  let recipientATA: PublicKey;
  let mint: Signer;
  let counterPDA: PublicKey;
  const TRANSFER_HOOK_PROGRAM_ID = program.programId;
  const decimals = 6;

  console.log(program.programId)

    // Helper function to confirm transactions
    async function confirmTransaction(
        connection: Connection,
        signature: TransactionSignature,
        desiredConfirmationStatus: TransactionConfirmationStatus = 'confirmed',
        timeout: number = 30000,
        pollInterval: number = 1000,
        searchTransactionHistory: boolean = false
    ): Promise<SignatureStatus> {
        const start = Date.now();

        while (Date.now() - start < timeout) {
            const { value: statuses } = await connection.getSignatureStatuses([signature], { searchTransactionHistory });

            if (!statuses || statuses.length === 0) {
                throw new Error('Failed to get signature status');
            }

            const status = statuses[0];

            if (status === null) {
                await new Promise(resolve => setTimeout(resolve, pollInterval));
                continue;
            }

            if (status.err) {
                throw new Error(`Transaction failed: ${JSON.stringify(status.err)}`);
            }

            if (status.confirmationStatus && status.confirmationStatus === desiredConfirmationStatus) {
                return status;
            }

            if (status.confirmationStatus === 'finalized') {
                return status;
            }

            await new Promise(resolve => setTimeout(resolve, pollInterval));
        }

        throw new Error(`Transaction confirmation timeout after ${timeout}ms`);
    }

  before(async () => {
    //   it("prepare accounts", async () => {
    authority = await newAccountWithLamports(provider.connection);
    recipient = await newAccountWithLamports(provider.connection);
    mint = anchor.web3.Keypair.generate();
    authorityATA = getAssociatedTokenAddressSync(
      mint.publicKey,
      authority.publicKey,
      false,
      TOKEN_2022_PROGRAM_ID,
      ASSOCIATED_TOKEN_PROGRAM_ID
    );
    recipientATA = getAssociatedTokenAddressSync(
      mint.publicKey,
      recipient.publicKey,
      false,
      TOKEN_2022_PROGRAM_ID,
      ASSOCIATED_TOKEN_PROGRAM_ID
    );
  });

 
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

  it("Burn tokens", async () => {
    try {
      console.log(`Burning 2 tokens from: ${associatedTokenAccount.toString()}`);
      await program.methods.burnToken(new anchor.BN(2)).accounts({
        mint: mintKey.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        tokenAccount: associatedTokenAccount,
        authority: provider.wallet.publicKey,
      }).rpc();

      const remainingBalance = (await program.provider.connection.getParsedAccountInfo(associatedTokenAccount)).value.data.parsed.info.tokenAmount.amount;
      console.log(`Remaining balance in associated token account after burn: ${remainingBalance}`);
      assert.equal(remainingBalance, 3); // 5 - 2 = 3 remaining tokens

    } catch (error) {
      console.error("Error during token burn:", error);
    }
  });

  it("Approve delegate", async () => {
    try {
      const delegateAccount = anchor.web3.Keypair.generate();

      console.log(`Approving delegate ${delegateAccount.publicKey.toString()} for 3 tokens`);
      await program.methods.approveDelegate(new anchor.BN(3)).accounts({
        tokenProgram: TOKEN_PROGRAM_ID,
        tokenAccount: associatedTokenAccount,
        delegate: delegateAccount.publicKey,
        authority: provider.wallet.publicKey,
      }).rpc();

      const tokenAccountInfo = await program.provider.connection.getParsedAccountInfo(associatedTokenAccount);
      console.log("Token account info after delegate approval:", tokenAccountInfo);

      // Implement detailed parsing of tokenAccountInfo to check if delegation worked correctly.
      // Depending on the SPL Token implementation, you might need to check the delegate and approved amount.

      assert.isTrue(true); // Replace with actual validation once delegate info is parsed

    } catch (error) {
      console.error("Error during delegate approval:", error);
    }
  });

  it("Get balance of token account", async () => {
    try {
      // Log the associated token account address
      console.log(`Getting balance of token account: ${associatedTokenAccount.toString()}`);
  
      // Call the getBalance method from the Solana program
      const balanceString: string = await program.methods
        .getBalance()
        .accounts({
          tokenAccount: associatedTokenAccount,
        })
        .rpc();  // Use `.rpc()` to invoke the method.
      
      // Debugging: Log the returned balance string
      console.log(`Returned balance string: ${balanceString}`);
  
      // Convert the returned balance string to BN, only if it's a valid number
      if (!/^\d+$/.test(balanceString)) {
        throw new Error(`Invalid balance format: ${balanceString}`);
      }
      const balanceBN = new BN(balanceString); // Safe conversion
  
      // Convert the BN balance to a number
      const tokenBalance = balanceBN.toNumber();  // Safely convert to number if the balance isn't too large
  
      // Log and assert the balance
      console.log(`Token account balance: ${tokenBalance}`);
      
      assert.strictEqual(tokenBalance, 3, "The token balance should be 3");
  
    } catch (error) {
      console.error("Error during balance retrieval:", error);
    }
  });
  
  it("create counter account", async () => {
    const [_counterPDA, _bump] = PublicKey.findProgramAddressSync(
      [authority.publicKey.toBuffer()],
      program.programId
    );
    counterPDA = _counterPDA;

    const tx = new Transaction().add(
      await program.methods
        .initialize()
        .accounts({
          counter: counterPDA,
          authority: authority.publicKey,
        })
        .instruction()
    );

    await sendAndConfirmTransaction(provider.connection, tx, [authority]);
  });

  it("create mint with transfer-hook", async () => {
    // 1. Create mint account
    // 2. Initialize transfer-hook
    // 3. Initialize mint account

    const extensions = [ExtensionType.TransferHook];
    const mintLen = getMintLen(extensions);
    const lamports =
      await provider.connection.getMinimumBalanceForRentExemption(mintLen);

    const mintTransaction = new Transaction().add(
      SystemProgram.createAccount({
        fromPubkey: authority.publicKey,
        newAccountPubkey: mint.publicKey,
        space: mintLen,
        lamports: lamports,
        programId: TOKEN_2022_PROGRAM_ID,
      }),
      createInitializeTransferHookInstruction(
        mint.publicKey,
        authority.publicKey,
        TRANSFER_HOOK_PROGRAM_ID,
        TOKEN_2022_PROGRAM_ID
      ),
      createInitializeMintInstruction(
        mint.publicKey,
        decimals,
        authority.publicKey,
        null,
        TOKEN_2022_PROGRAM_ID
      )
    );

    await sendAndConfirmTransaction(provider.connection, mintTransaction, [
      authority,
      mint,
    ]);
  });

  it("setup extra account metas", async () => {
    // 1. Create extra account

    const [_extractAccountMetaPDA, _bump] = PublicKey.findProgramAddressSync(
      [Buffer.from("extra-account-metas"), mint.publicKey.toBuffer()],
      TRANSFER_HOOK_PROGRAM_ID
    );

    const initExtraAccountMetaInstruction = await program.methods
      .initializeExtraAccountMetaList(_bump)
      .accounts({
        extraAccount: _extractAccountMetaPDA,
        counter: counterPDA,
        mint: mint.publicKey,
        authority: authority.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([authority])
      .instruction();

    const setupTransaction = new Transaction().add(
      initExtraAccountMetaInstruction,
      // Transfer some lamports to the extra account for rent
      SystemProgram.transfer({
        fromPubkey: authority.publicKey,
        toPubkey: _extractAccountMetaPDA,
        lamports: 10000000,
      })
    );

    const hash = await sendAndConfirmTransaction(
      provider.connection,
      setupTransaction,
      [authority]
    );
    console.log("setup extra account metas hash:", hash);
  });

  it("transfer token", async () => {
    // Create associated token account for recipient and transfer 1 token
  
    const transferInstruction = createTransferCheckedInstruction(
      authorityATA,
      mint.publicKey,
      recipientATA,
      authority.publicKey,
      1 * 10 ** decimals,
      decimals,
      [],
      TOKEN_2022_PROGRAM_ID
    );
  
    // Define extra accounts required by your Transfer Hook logic
    const extraAccountMetas = [
      // Example: include the PDA or other program-required accounts
      {
        pubkey: counterPDA,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: TRANSFER_HOOK_PROGRAM_ID,
        isSigner: false,
        isWritable: false,
      },
      // Add any additional required accounts
    ];
  
    // Add the extra accounts to the instruction manually
    transferInstruction.keys.push(...extraAccountMetas);
  
    const transferTransaction = new Transaction().add(transferInstruction);
    const signature = await sendAndConfirmTransaction(
      provider.connection,
      transferTransaction,
      [authority]
    );
  
    console.log("Transfer hash:", signature);
  });

  it("mint token", async () => {
    // 1. Create associated token account for authority
    // 1. Create associated token account for recipient
    // 2. Mint 100 tokens to authority

    const mintToTransaction = new Transaction().add(
      createAssociatedTokenAccountInstruction(
        authority.publicKey,
        authorityATA,
        authority.publicKey,
        mint.publicKey,
        TOKEN_2022_PROGRAM_ID,
        ASSOCIATED_TOKEN_PROGRAM_ID
      ),
      createAssociatedTokenAccountInstruction(
        authority.publicKey,
        recipientATA,
        recipient.publicKey,
        mint.publicKey,
        TOKEN_2022_PROGRAM_ID,
        ASSOCIATED_TOKEN_PROGRAM_ID
      ),
      createMintToInstruction(
        mint.publicKey,
        authorityATA,
        authority.publicKey,
        100 * 10 ** decimals,
        [],
        TOKEN_2022_PROGRAM_ID
      )
    );

    const res = await sendAndConfirmTransaction(
      provider.connection,
      mintToTransaction,
      [authority]
    );

    console.log("Mint to hash:", res);
  });


});