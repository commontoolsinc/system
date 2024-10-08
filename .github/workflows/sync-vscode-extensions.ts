#!/usr/bin/env -S npx -y tsx --no-cache

// @ts-expect-error
import * as fs from 'fs';

interface DevContainer {
	customizations: {
		vscode: {
			extensions: string[];
		};
	};
}

interface VSCodeExtensions {
	recommendations: string[];
}

/**
 * Detect the indentation used in a JSON string.
 * @param json JSON string.
 * @param fallback Fallback indentation.
 * @returns Indentation used in the JSON string.
 */
function detectJsonIndentation(json: string, fallback: string = "\t"): string | number {
	const spaces = json.match(/^(\s+)/);
	if (spaces) {
		const tabs = spaces[1].match(/\t/g);
		return tabs ? "\t" : spaces[1].length;
	}
	return fallback;

}

/**
 * Read and parse a JSON file, removing comments.
 * @param path Path to the JSON file.
 * @returns Parsed JSON data and indentation.
 */
function readCleanJson<T>(path: string): { data: T, indentation: string | number; } {
	const file = fs.readFileSync(path, "utf8");
	const stripForwardSlashForwardSlashComment = new RegExp('//(.*)', 'g');
	const stripForwardSlashStarComment = new RegExp('[/][*](.*)[*][/]', 'gm');

	const indentation = detectJsonIndentation(file);

	const cleanedFile = file.replace(stripForwardSlashForwardSlashComment, '').replace(stripForwardSlashStarComment, '');

	return {
		data: JSON.parse(cleanedFile),
		indentation
	};
}

/**
 * Combine and deduplicate extension lists.
 * @param extensionLists List of extension lists.
 * @returns Combined and deduplicated extension list.
 */
function unionOfStringLists(extensionLists: string[][]): string[] {
	return [...new Set(extensionLists.flat())].sort(
		(a, b) => a.localeCompare(b, undefined, { sensitivity: "base" })
	);
}

/**
 * Synchronize the extensions in devcontainer.json and extensions.json files.
 */
function syncExtensions(
	devcontainerFilePath: string = ".devcontainer/devcontainer.json",
	extensionsFilePath: string = ".vscode/extensions.json"
): void {
	// Read devcontainer.json
	const { data: devcontainer, indentation: devcontainerIndentation } = readCleanJson<DevContainer>(devcontainerFilePath);
	const devcontainerExtensions = devcontainer.customizations.vscode.extensions || [];
	console.info("devcontainer.json extensions: ", devcontainerExtensions);

	// Read extensions.json
	const { data: vscodeExtensions, indentation: vscodeExtensionsIndentation } = readCleanJson<VSCodeExtensions>(extensionsFilePath);
	const recommendations = vscodeExtensions.recommendations || [];
	console.info("extensions.json recommendations: ", recommendations);

	// Combine and deduplicate extensions
	const combinedExtensions = unionOfStringLists([devcontainerExtensions, recommendations]);

	console.info("Combined extensions: ", combinedExtensions);

	// Update devcontainer.json
	devcontainer.customizations.vscode.extensions = combinedExtensions;
	fs.writeFileSync(".devcontainer/devcontainer.json", JSON.stringify(devcontainer, null, devcontainerIndentation));

	// Update extensions.json
	vscodeExtensions.recommendations = combinedExtensions;
	fs.writeFileSync(".vscode/extensions.json", JSON.stringify(vscodeExtensions, null, vscodeExtensionsIndentation
	));

	console.log("VSCode extensions synchronized successfully.");
}

//@ts-ignore
const repoRoot = process.argv[2];
if (repoRoot) {
	//@ts-ignore
	process.chdir(repoRoot);
}
syncExtensions();
