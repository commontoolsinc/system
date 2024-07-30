import { write } from "common:io/state@0.0.1";

export const run = () => {
  let meta = JSON.stringify(Object.getOwnPropertyNames(import.meta).sort());
  write("output", {
    tag: "string",
    val: meta,
  });
};