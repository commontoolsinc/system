import Anthropic from "@anthropic-ai/sdk";
import { makeTool } from "../tool";
import { defineModule, ModuleShape } from "./definition";
import { ESM_SH_README } from "./esmsh";
import { JavaScriptModuleDefinition } from "common-runtime";

export const JAVASCRIPT_SYSTEM_PROMPT = `
You are a programmer. Your job is to help the user produce JavaScript programs that operate on a very specific API.

The user will provide you with two inputs:

1. Spec: a prose description of the program
2. Schema: a description of the programs inputs and outputs

To access the special API, you can import the 'common:io/state@0.0.1' module. This module provides two functions: 'read' and 'write'.

Here is an example schema:

\`\`\`json
{
    "description": "A contrived program",
    "inputs": [
        {
            "name": "foo",
            "description": "A contrived input",
            "valueKind": "string"
        }
    ],
    "outputs": [
        {
            "name": "bar",
            "description": "A contrived output",
            "valueKind": "string"
        }
    ]
}
\`\`\`

Here is an example program:

\`\`\`js
import { read, write } from "common:io/state@0.0.1";

export const run = () => {
  const foo = read("foo");
  const value = foo?.deref();

  write("bar", {
    tag: "string",
    val: \`\${value?.val}:bar\`,
  });
};
\`\`\`

JavaScript programs are always expressed as self-contained standard ECMAScript Modules.

The exported \`run\` function should always execute synchronously (do not use \`async\` or return a \`Promise\`)

The API that is exported by \`common:io/state@0.0.1\` is as follows:

\`\`\`ts
type Value = {
  tag: 'string'|'boolean'|'number'|'buffer;
  val: string|boolean|number|Uint8Array
};

interface Reference {
    deref(): Value|undefined
}

type read = (string) => void|Reference;
type write = (string, Value) => void;
\`\`\`

If you use external dependencies, they must be imported from esm.sh. The usage documentation for esm.sh follows:

${ESM_SH_README}
`;

export const EVAL_JAVASCRIPT_TOOL: Anthropic.Messages.Tool = {
    "name": "eval_javascript",
    "description": "Produce a JavaScript module given a program spec and a schema",
    "input_schema": {
        "type": "object",
        "properties": {
            "sourceCode": {
                "description": "The source code for the JavaScript module",
                "type": "string"
            }
        },
        "required": ["sourceCode"]
    }
};

export type SourceCode = {
    sourceCode: string
};

export const getJsModule = makeTool<SourceCode>(JAVASCRIPT_SYSTEM_PROMPT, EVAL_JAVASCRIPT_TOOL);

export interface FunctionDetails {
    definition: ModuleShape;
    sourceCode: string;
}

export const specToFunction = async (spec: string): Promise<FunctionDetails> => {
    let definition = await defineModule(spec);

    let composedPrompt = `
Spec:

${spec}

Schema:

${JSON.stringify(definition, null, 2)}`;

    return {
        definition,
        sourceCode: (await getJsModule(composedPrompt))?.sourceCode
    };
};

(self as any).specToFunction = specToFunction;