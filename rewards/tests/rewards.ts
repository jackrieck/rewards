import * as anchor from "@project-serum/anchor";
import * as splToken from "@solana/spl-token";
import * as mplMd from "@metaplex-foundation/mpl-token-metadata";
import { Program } from "@project-serum/anchor";
import { Rewards } from "../target/types/rewards";
import { RewardsTester } from "../target/types/rewards_tester";
import { assert } from "chai";

describe("rewards", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const rewardsProgram = anchor.workspace.Rewards as Program<Rewards>; 
  const rewardsTesterProgram = anchor.workspace.RewardsTester as Program<RewardsTester>;

  const wallet = (rewardsTesterProgram.provider as anchor.AnchorProvider).wallet;

  it("create a simple plan", async () => {
    let rewardPlanName = "discount";

    let createRewardPlanParams = {
      name: rewardPlanName,
      threshold: new anchor.BN(1),
      allowed_program: rewardsTesterProgram.programId,
      metadataUri: "https://foo.com/bar.json",
      metadataSymbol: "REWARDS",
    };

    // holds all the reward plan configuration
    let [rewardPlanConfig, _rewardPlanConfigBump] =
      anchor.web3.PublicKey.findProgramAddressSync(
        [wallet.publicKey.toBuffer(), Buffer.from(rewardPlanName)],
        rewardsProgram.programId
      );

    // create mint using reward config as the seed
    let [mint, _mintBump] = anchor.web3.PublicKey.findProgramAddressSync(
      [rewardPlanConfig.toBuffer()],
      rewardsProgram.programId
    );

    // derive metaplex metadata account with seeds it expects
    let [metadata, _metadataBump] =
      await anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from("metadata"), mplMd.PROGRAM_ID.toBuffer(), mint.toBuffer()],
        mplMd.PROGRAM_ID
      );

    const createRewardPlanTxSig = await rewardsProgram.methods
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

    const customer = await initWallet(rewardsProgram.provider.connection); 

    const customerAta = await splToken.getAssociatedTokenAddress(mint, customer.publicKey);

    // call buy, which calls Reward via CPI
    let buyTxSig = await rewardsTesterProgram.methods.buy(rewardPlanName, wallet.publicKey).accounts({
      mint: mint,
      config: rewardPlanConfig,
      user: customer.publicKey,
      userAta: customerAta,
      systemProgram: anchor.web3.SystemProgram.programId,
      rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      tokenProgram: splToken.TOKEN_PROGRAM_ID,
      associatedTokenProgram: splToken.ASSOCIATED_TOKEN_PROGRAM_ID,
      rewardProgram: rewardsProgram.programId,
    }).signers([customer]).rpc();
    console.log("buyTxSig: %s", buyTxSig);

    await delay(1000);

    // call buy, which calls Reward via CPI
    buyTxSig = await rewardsTesterProgram.methods.buy(rewardPlanName, wallet.publicKey).accounts({
      mint: mint,
      config: rewardPlanConfig,
      user: customer.publicKey,
      userAta: customerAta,
      systemProgram: anchor.web3.SystemProgram.programId,
      rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      tokenProgram: splToken.TOKEN_PROGRAM_ID,
      associatedTokenProgram: splToken.ASSOCIATED_TOKEN_PROGRAM_ID,
      rewardProgram: rewardsProgram.programId,
    }).signers([customer]).rpc();
    console.log("buyTxSig: %s", buyTxSig);
  });
});

// create a new wallet and seed it with lamports
async function initWallet(
  connection: anchor.web3.Connection
): Promise<anchor.web3.Keypair> {
  const wallet = anchor.web3.Keypair.generate();

  const airdropTxSig = await connection.requestAirdrop(
    wallet.publicKey,
    100_000_000_000
  );
  await connection.confirmTransaction(airdropTxSig, "confirmed");

  return wallet;
}

async function delay(ms: number) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}
