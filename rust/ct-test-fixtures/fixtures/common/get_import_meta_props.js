import { write } from "common:io/state@0.0.1";

export const run = () => {
  const importMetaSignature = [];

  for (key of Object.getOwnPropertyNames(import.meta).sort()) {
    importMetaSignature.push(key, String(import.meta[key]));
  }

  write("output", {
    tag: "string",
    val: JSON.stringify(importMetaSignature),
  });
};
