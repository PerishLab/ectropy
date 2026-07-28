import { betaAboveStable } from "../metadata/beta.ts";

Deno.test("beta base must be newer than stable", () => {
  if (betaAboveStable("0.1.0", "0.1.0")) {
    throw new Error("equal beta base passed the stable floor");
  }
  if (betaAboveStable("0.0.9", "0.1.0")) {
    throw new Error("older beta base passed the stable floor");
  }
  if (!betaAboveStable("0.1.1", "0.1.0")) {
    throw new Error("newer beta base failed the stable floor");
  }
});
