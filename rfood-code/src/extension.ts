// The module 'vscode' contains the VS Code extensibility API
// Import the module and reference it with the alias vscode in your code below
import * as vscode from 'vscode';
import * as rfood from 'rfood';
import { transcode } from 'buffer';
import { start } from 'repl';

function transform(context: vscode.ExtensionContext, isOOP: boolean) {
	// Get the file path
	const editorWindow = vscode.window.activeTextEditor;
	const editor = vscode.window.activeTextEditor;
	if (editorWindow === undefined || editor === undefined) {
		console.log('Please open the file you would like to transform');
		return;
	}

	let file = editorWindow.document;
	vscode.window.showInformationMessage("Transforming " + file.fileName);

	console.log("About to transform ", file);
	try {
		let result = rfood.transform_string_binding(file.getText(), isOOP);
		console.log("Transformig completed result: ", result);

		var firstLine = editor.document.lineAt(0);
		var lastLine = editor.document.lineAt(editor.document.lineCount - 1);
		var textRange = new vscode.Range(firstLine.range.start, lastLine.range.end);

		editor.edit(editBuilder => {
			editor.selections.forEach(sel => {
				file.lineCount
				// editBuilder.replace(vscode.Range(startLine: 0))
				editBuilder.delete(textRange);
				editBuilder.insert(firstLine.range.start, result);
			});
		});

		// Format the file
		vscode.commands.executeCommand('editor.action.formatDocument');
		
	} catch (error) {
		console.log("Transformig failed ", error);
	}	
	
}

// this method is called when your extension is activated
// your extension is activated the very first time the command is executed
export function activate(context: vscode.ExtensionContext) {
	
	// Use the console to output diagnostic information (console.log) and errors (console.error)
	// This line of code will only be executed once when your extension is activated
	console.log('Congratulations, your extension "rfood-code" is now active!');

	// The command has been defined in the package.json file
	// Now provide the implementation of the command with registerCommand
	// The commandId parameter must match the command field in package.json
	let helloWorld = vscode.commands.registerCommand('rfood-code.helloWorld', () => {
		// The code you place here will be executed every time your command is executed
		// Display a message box to the user
		// vscode.window.showInformationMessage('Hello World from rfood-code!');
		vscode.window.showInformationMessage(rfood.hello_world("James"));
	});

	let transOOP = vscode.commands.registerCommand('rfood-code.transOOP', () => {
		transform(context, true);
	});

	let transFP = vscode.commands.registerCommand('rfood-code.transFP', () => {
		transform(context, false);
	});

	context.subscriptions.push(helloWorld);
	context.subscriptions.push(transOOP);
	context.subscriptions.push(transFP);
}

// this method is called when your extension is deactivated
export function deactivate() {}
