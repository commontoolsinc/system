#!/usr/bin/env bash

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
TYPESCRIPT_DIR="$SCRIPT_DIR/../typescript"

declare -a WASM_COMPONENTS=("rust/common-javascript-interpreter" "rust/common-formula-javascript-interpreter" "rust/ct-js-vm")
declare -a WITS=("data" "io" "function" "formula" "basic")

print_help() {
    echo "WIT tools for Common System"
    echo ""
    echo "Usage: wit-tools [COMMAND]"
    echo ""
    echo "Commands:"
    echo "  deps     Update dependency references via \`wit-deps\`"
    echo "           for definitions and WASM artifacts."
    echo "  types <WIT-NAME> <OUT_DIR>"
    echo "           Generates TypeScript types for WIT-NAME."
    echo "  lint     Lint wit files via \`wasm-tools\`."
    echo "  clean    Clean all generated files created by this tool."
    echo "  help     Print this message."
    exit 0
}

update_deps() {
    for wit in "${WITS[@]}"
    do
        cd "$SCRIPT_DIR/common/$wit" && wit-deps && cd - > /dev/null
    done
    for component in "${WASM_COMPONENTS[@]}"
    do
        cd "$SCRIPT_DIR/../$component" && wit-deps && cd - > /dev/null
    done
}

gen_types() {
    wit_name="$1"
    wit_dir="$SCRIPT_DIR/common/$wit_name/wit"
    output_dir="$2"
    mkdir -p "$output_dir"
    if [ "$wit_name" = "function" ]; then
        jco types -o $output_dir --world-name module --name index $wit_dir
    else
        jco types -o $output_dir --name index $wit_dir
    fi
}

lint_wit() {
    for wit in "${WITS[@]}"
    do
        error="$(wasm-tools component wit "$SCRIPT_DIR/common/$wit/wit")"
        ret=$?
        if [ $ret -ne 0 ]; then
            echo "$error"
	    exit $ret
	fi
    done
}

clean_artifacts() {
    for wit in "${WITS[@]}"
    do
        rm -rf "$SCRIPT_DIR/common/$wit/wit/deps"
    done
}

case "$1" in
  deps)
    update_deps
    ;;
  types)
    gen_types "$2" "$3"
    ;;
  lint)
    lint_wit
    ;;
  clean)
    clean_artifacts 
    ;;
  *)
    print_help
    ;;
esac
