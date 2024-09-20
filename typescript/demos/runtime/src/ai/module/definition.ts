import Anthropic from "@anthropic-ai/sdk";
import { makeTool } from "../tool.js";

export const MODULE_DEFINITION_SYSTEM_PROMPT = `
You are a programmer.

Your job is to review a program spec, and produce a JSON descriptor of the program's inputs and outputs.

Whenever you refer to the program, refer to it as a spell.

For example:

Spec: "A program that transforms a PDF into markdown"
Descriptor:

\`\`\`json
{
    "description": "A program that transforms a PDF into markdown",
    "inputs": [
        {
            "name": "pdf",
            "description": "The raw bytes of the PDF to convert",
            "valueKind": "buffer"
        }
    ],
    "outputs": [
        {
            "name": "markdown",
            "description": "The markdown representation of text content extracted from the PDF",
            "valueKind": "string"
        }
    ]
}
\`\`\`

Note that there may be multiple inputs and/or multiple outputs!

All inputs and outputs should specify one of the following "valueKind":

- \`"string"\`: A UTF-8 string
- \'"number"\": A double-precision floating point number
- \'"boolean"\": A true or false value
- \'"buffer"\": A contiuous array of bytes

If the return type is expected to be JSON, then the "valueKind" should be string.
If the return type is expected to be binary data (such as an image), then the "valueKind" should be a buffer.
`;

const DEFINE_MODULE_TOOL: Anthropic.Messages.Tool = {
    "name": "define_module",
    "description": "Produce a module definition for a given program spec",
    "input_schema": {
        "type": "object",
        "properties": {
            "description": {
                "type": "string"
            },
            "inputs": {
                "type": "array",
                "description": "The granular inputs aka parameters to the program, as well as the their type",
                "items": {
                    "type": "object",
                    "properties": {
                        "name": {
                            "type": "string"
                        },
                        "description": {
                            "description": "A detailed description of this input",
                            "type": "string"
                        },
                        "valueKind": {
                            "description": "The type that corresponds to this input",
                            "type": "string",
                            "enum": ["string", "number", "boolean", "buffer"]
                        }
                    },
                    "required": ["name", "valueKind"]
                }
            },
            "outputs": {
                "type": "array",
                "description": "The discrete outputs of the program, similar conceptually to the return value of a function",
                "items": {
                    "type": "object",
                    "properties": {
                        "name": {
                            "type": "string"
                        },
                        "description": {
                            "description": "A detailed description of this output",
                            "type": "string"
                        },
                        "valueKind": {
                            "description": "The type that corresponds to this output",
                            "type": "string",
                            "enum": ["string", "number", "boolean", "buffer"]
                        }
                    },
                    "required": ["name", "valueKind"]
                }
            }
        },
        "required": ["description", "inputs", "outputs"]
    }
};

export interface IoShape {
    name: 'string',
    description: 'string',
    valueKind: 'number' | 'boolean' | 'string' | 'buffer'
}

export interface ModuleShape {
    description: 'string',
    inputs: IoShape[],
    outputs: IoShape[]
}

type SourceCode = string;

export const defineModule = makeTool<ModuleShape>(MODULE_DEFINITION_SYSTEM_PROMPT, DEFINE_MODULE_TOOL);