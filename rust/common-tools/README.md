# common-tools

CLI tools for the Common Runtime.

## Usage

First, start the runtime server. This will run until the process is terminated.

```bash
$ ct serve
```

Create a file "reflect.js" that passes its input named `"input"` to the output named `"output"`:

```js
import { read, write } from "common:io/state@0.0.1";

export const run = () => {
  const input = read("input");
  const value = input?.deref().val;

  write("output", {
    tag: "string",
    val: value,
  });
};
```

Execute the module passing stdin to `"input"`, reflecting to `"output"`, to stdout:

```bash
$ printf "Hello!" | ct run reflect.js -i
Hello!
```