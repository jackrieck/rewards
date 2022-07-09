import * as anchor from "@project-serum/anchor";
import * as splToken from "@solana/spl-token";
import * as mplMd from "@metaplex-foundation/mpl-token-metadata";
import { Program } from "@project-serum/anchor";
import { Rewards } from "../target/types/rewards";
import { assert } from "chai";

describe("rewards", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.Rewards as Program<Rewards>;
  const wallet = (program.provider as anchor.AnchorProvider).wallet;

  it("create a simple plan", async () => {
    let rewardPlanName = "discount";

    let createRewardPlanParams = {
      name: rewardPlanName,
      threshold: new anchor.BN(1),
      rewardProgramId: splToken.TOKEN_PROGRAM_ID,
      rewardProgramIxAccountsLen: new anchor.BN(3),
      metadataUri: "https://foo.com/bar.json",
      metadataSymbol: "REWARDS",
    };

    // holds all the reward plan configuration
    let [rewardPlanConfig, _rewardPlanConfigBump] =
      anchor.web3.PublicKey.findProgramAddressSync(
        [wallet.publicKey.toBuffer(), Buffer.from(rewardPlanName)],
        program.programId
      );

    // create mint using reward config as the seed
    let [mint, _mintBump] = anchor.web3.PublicKey.findProgramAddressSync(
      [rewardPlanConfig.toBuffer()],
      program.programId
    );

    // derive metaplex metadata account with seeds it expects
    let [metadata, _metadataBump] =
      await anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from("metadata"), mplMd.PROGRAM_ID.toBuffer(), mint.toBuffer()],
        mplMd.PROGRAM_ID
      );

    const createRewardPlanTxSig = await program.methods
      .createRewardPlan(createRewardPlanParams)
      .accounts({
        mint: mint,
        metadata: metadata,
        config: rewardPlanConfig,
        admin: wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        tokenProgram: splToken.TOKEN_PROGRAM_ID,
        associatedTokenProgram: splToken.ASSOCIATED_TOKEN_PROGRAM_ID,
        metadataProgram: mplMd.PROGRAM_ID,
      })
      .rpc();
    console.log("createRewardPlan txSig: %s", createRewardPlanTxSig);

    const userAta = await splToken.getAssociatedTokenAddress(
      mint,
      wallet.publicKey
    );

    const bogusTransferIx = splToken.createTransferInstruction(
      userAta,
      userAta,
      wallet.publicKey,
      0
    );

    let approveParams = {
      name: rewardPlanName,
      admin: wallet.publicKey,
    };

    let approveIx = await program.methods
      .approve(approveParams)
      .accounts({
        mint: mint,
        metadata: metadata,
        config: rewardPlanConfig,
        instructions: anchor.web3.SYSVAR_INSTRUCTIONS_PUBKEY,
        user: wallet.publicKey,
        userAta: userAta,
        systemProgram: anchor.web3.SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        tokenProgram: splToken.TOKEN_PROGRAM_ID,
        associatedTokenProgram: splToken.ASSOCIATED_TOKEN_PROGRAM_ID,
      })
      .instruction();

    let tx = new anchor.web3.Transaction();
    tx.add(approveIx, bogusTransferIx);
    let txSig = await program.provider.sendAndConfirm!(tx, [], {
      commitment: "confirmed",
    });
    console.log("approve + transfer txSig: %s", txSig);

    // check that user received the rewards token
    let balance = (
      await program.provider.connection.getTokenAccountBalance(
        userAta,
        "confirmed"
      )
    ).value.amount;
    assert.strictEqual(Number(balance), 1);

    // wait for new block
    await delay(500);

    tx = new anchor.web3.Transaction();
    tx.add(approveIx, bogusTransferIx);
    txSig = await program.provider.sendAndConfirm!(tx, [], {
      commitment: "confirmed",
    });
    console.log("approve + transfer txSig: %s", txSig);

    // check that user received the rewards token
    // balance should be the same as before because it met the threshold and 1 was burned
    balance = (
      await program.provider.connection.getTokenAccountBalance(
        userAta,
        "confirmed"
      )
    ).value.amount;
    assert.strictEqual(Number(balance), 1);
  });
});

async function delay(ms: number) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}
