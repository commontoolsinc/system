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

function detectJsonIndentation(json: string, fallback: string = "\t"): string | number {
	const spaces = json.match(/^(\s+)/);
	if (spaces) {
		const tabs = spaces[1].match(/\t/g);
		return tabs ? "\t" : spaces[1].length;
	}
	return fallback;

}

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

function syncExtensions(): void {
	// Read devcontainer.json
	const { data: devcontainer, indentation: devcontainerIndentation } = readCleanJson<DevContainer>(".devcontainer/devcontainer.json");
	const devcontainerExtensions = devcontainer.customizations.vscode.extensions || [];
	console.info("devcontainer.json extensions: ", devcontainerExtensions);

	// Read extensions.json
	const { data: vscodeExtensions, indentation: vscodeExtensionsIndentation } = readCleanJson<VSCodeExtensions>(".vscode/extensions.json");
	const recommendations = vscodeExtensions.recommendations || [];
	console.info("extensions.json recommendations: ", recommendations);

	// Combine and deduplicate extensions
	const combinedExtensions = [...new Set([...devcontainerExtensions, ...recommendations])]
		.sort((a, b) => a.localeCompare(b, undefined, { sensitivity: 'base' }));

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

syncExtensions();
