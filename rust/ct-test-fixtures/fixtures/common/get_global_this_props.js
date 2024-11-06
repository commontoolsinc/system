import { write } from "common:io/state@0.0.1";

export const run = () => {
  let globals = JSON.stringify(Object.getOwnPropertyNames(globalThis).sort());
  write("output", {
    tag: "string",
    val: globals,
  });
};