import { LitElement, css, html } from 'lit'
import { customElement, property, query, state } from 'lit/decorators.js'
import init, { CommonFunction, CommonRuntime, JavaScriptModuleDefinition, JavaScriptValueMap } from 'common-runtime/common_runtime.js';
import callouts from './callouts.js';

import { specToFunction, FunctionDetails } from './ai/module/javascript.js';

import "@shoelace-style/shoelace/dist/themes/dark.css";
import "@shoelace-style/shoelace";
import { SlTextarea } from '@shoelace-style/shoelace';

await init();

const defaultForValueType = (valueType: "string" | "number" | "boolean" | "buffer") => {
  switch (valueType) {
    case 'string': return "";
    case 'number': return 0;
    case 'boolean': return true;
    case 'buffer': return new Uint8Array();
  }
}

const valueForKind = (kind: "string" | "number" | "boolean" | "buffer", value: any) => {
  switch (kind) {
    case 'string': return value + '';
    case 'number': return Number(value);
    case 'boolean': return !!value;
    case 'buffer': return value instanceof Uint8Array ? value : new Uint8Array();
  }
}

const inputForValue = (name: string, valueType: "string" | "number" | "boolean" | "buffer") => {
  switch (valueType) {
    case 'string': return html`<sl-input class="input" data-kind="${valueType}" data-name="${name}" type="text" value="${defaultForValueType(valueType)}"></sl-input>`;
    case 'number': return html`<sl-input class="input" data-kind="${valueType}" data-name="${name}" type="number" value="${defaultForValueType(valueType)}"></sl-input>`;
    case 'boolean': return html`<sl-input class="input" data-kind="${valueType}" data-name="${name}" type="checkbox" ?checked=${defaultForValueType(valueType)}></sl-input>`;
    case 'buffer': return html`<input class="input" data-kind="${valueType}" data-name="${name}" type="file"></input>`;
  }
}

const functionDetailsToModuleDefinition = (details: FunctionDetails): JavaScriptModuleDefinition => {
  const inputs: JavaScriptModuleDefinition['inputs'] = {};
  const outputs: JavaScriptModuleDefinition['outputs'] = {};

  for (const input of details.definition.inputs) {
    console.log(input);
    inputs[input.name] = {
      tag: input.valueKind,
      val: defaultForValueType(input.valueKind)
    };
  }

  for (const output of details.definition.outputs) {
    outputs[output.name] = output.valueKind;
  }

  return {
    inputs,
    outputs,
    body: details.sourceCode
  }
}




@customElement('runtime-demo')
export class RuntimeDemo extends LitElement {

  @state()
  loading = false;

  @query("#spec")
  specTextArea?: SlTextarea;

  @query("#form")
  form?: Element;

  #rt = new CommonRuntime("http://localhost:8081");

  @state()
  error?: String;

  @state()
  details?: FunctionDetails;

  @state()
  definition?: JavaScriptModuleDefinition;

  @state()
  function?: CommonFunction;

  @state()
  outputs?: JavaScriptValueMap;

  render() {
    let step = 1;

    const formControls = [];
    let outputs = html``;
    let error = html``;

    if (this.error) {
      error = html`
        <div class="callout callout--error">
          Uh oh..... ${this.error}
        </div>
      `;
    } else if (this.details != null && this.definition != null) {
      step = 2;

      for (const input of this.details.definition.inputs) {
        formControls.push(html`
          <li>
            <span class="name">${input.name}</span>
            <span class="description">${input.description}</span>
            <div class="input-wrapper">
              ${inputForValue(input.name, input.valueKind)}
            </div>
          </li>
        `);
      }

      if (this.outputs != null) {
        const items = [];
        for (let [key, value] of Object.entries(this.outputs)) {
          items.push(html`<li>
            <span class="output-name">${key} (${value.tag}):</span><span class="output-value">${value.val}</span>
          </li>`);
        }

        outputs = html`<ul>
          ${items}
        </ul>`;
      }
    }

    return html`
      <section id="flow" class="step${step}">
        <section id="loading" ?hidden=${!this.loading}>
          <img src="/ferris.gif">
        </section>
        <section id="prompt" ?hidden=${this.loading}>
          ${error}
          <div class="callout callout--tip">
            Enter a prompt
          </div>
          <sl-textarea id="spec" value="Add two numbers together">
          </sl-textarea>
          <sl-button @click=${() => this.#generateModule()}>
            Make it so!
          </sl-button>
        </section>
        <section id="form" ?hidden=${this.loading}>
          <ul>
            ${formControls}
          </ul>
          <div class="actions">
            <sl-button @click=${() => this.#reset()}>Start over</sl-button>
            <sl-button @click=${() => this.#runModule()}>Run</sl-button>
          </div>
          <sl-divider></sl-divider>
          <div class="callout">
            Outputs will appear below
          </div>
          ${outputs}
        </section>
      </section>
    `;
  }

  #reset = () => {
    this.details = undefined;
    this.definition = undefined;
    this.outputs = undefined;
    this.error = undefined;
    this.loading = false;
  }

  #generateModule = async () => {
    let prompt = this.specTextArea?.value;

    if (!prompt) {
      return;
    }

    this.#reset();


    this.loading = true;

    try {
      console.log(prompt);
      const details = await specToFunction(prompt);
      console.log(details);

      let moduleDefinition = functionDetailsToModuleDefinition(details)
      console.log(moduleDefinition);

      this.function = await this.#rt.instantiate(moduleDefinition);
      this.details = details;
      this.definition = moduleDefinition;
    } catch (error) {
      this.error = error?.toString() || "Something bad happened but... what: I'm not sure";
    }

    this.loading = false;
  };

  #collectInputs = (): JavaScriptValueMap => {
    let inputs = (Array.prototype.slice.call(this.form?.querySelectorAll(".input")) || []) as HTMLInputElement[];
    let result: JavaScriptValueMap = {};

    console.log(this.form, inputs);

    for (let input of inputs) {
      let name = input.dataset.name || "NAME_MISSING";
      let kind = input.dataset.kind || "KIND_MISSING";

      result[name] = {
        tag: kind,
        val: valueForKind(kind as any, input.value)
      };
    }

    return result;
  }

  #runModule = async () => {
    let inputs = this.#collectInputs();
    console.log("INPUT:", inputs);

    let output = await this.function?.run(inputs);

    console.log("OUTPUT:", output);

    this.outputs = output;
  };

  static styles = [
    callouts,
    css`
    #flow {
      display: flex;
      flex-direction: column;
      width: 640px;
      padding-top: 1em;
    }

    [hidden] {
      display: none !important;
    }

    #loading {
      display: flex;
      align-items: center;
      justify-content: center;
      width: 100%;
      height: 480px;
    }

    * {
      box-sizing: border-box;
    }

    #flow:not(.step1) #prompt {
      display: none;
    }

    #flow:not(.step2) #form {
      display: none;
    }

    #spec {
      margin-bottom: 1em;
    }

    ul {
      gap: 1em;
    }

    ul, li {
      display: flex;
      flex-direction: column;

      margin: 0;
      padding: 0;
    }

    li {
      display: flex;
    }

    .name {
      font-weight: bold;
    }

    .description {
      font-style: italic;
    }

    .actions {
      margin: 1em 0;
      display: flex;
      flex-direction: row;
      justify-content: flex-end;
      gap: 1em;
    }
    `
  ];
}


declare global {
  interface HTMLElementTagNameMap {
    'runtime-demo': RuntimeDemo
  }
}
