import * as anchor from "@project-serum/anchor";
import * as splToken from "@solana/spl-token";
import * as mplMd from "@metaplex-foundation/mpl-token-metadata";
import { Program } from "@project-serum/anchor";
import { Rewards } from "../target/types/rewards";
import { metadataBeet } from "@metaplex-foundation/mpl-token-metadata";

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
    let [mint, _mintBump] =
      anchor.web3.PublicKey.findProgramAddressSync(
        [rewardPlanConfig.toBuffer()],
        program.programId
      );

    // derive metaplex metadata account with seeds it expects
    let [metadata, _metadataBump] =
      await anchor.web3.PublicKey.findProgramAddressSync(
        [
          Buffer.from("metadata"),
          mplMd.PROGRAM_ID.toBuffer(),
          mint.toBuffer(),
        ],
        mplMd.PROGRAM_ID
      );

    const txSig = await program.methods
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
    console.log("createRewardPlan txSig: %s", txSig);
  });
});
