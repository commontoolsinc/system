import { read, write } from "common:io/state@0.0.1";

export const run = () => {
  const foo = read("foo");
  const value = foo?.deref();

  write("bar", {
    tag: "string",
    val: `${value}:bar`,
  });
};
