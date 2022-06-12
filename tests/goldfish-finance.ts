import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { GoldfishFinance } from "../target/types/goldfish_finance";

describe("goldfish-finance", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.GoldfishFinance as Program<GoldfishFinance>;

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  });
});
