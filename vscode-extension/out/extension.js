"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || function (mod) {
    if (mod && mod.__esModule) return mod;
    var result = {};
    if (mod != null) for (var k in mod) if (k !== "default" && Object.prototype.hasOwnProperty.call(mod, k)) __createBinding(result, mod, k);
    __setModuleDefault(result, mod);
    return result;
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.deactivate = exports.activate = void 0;
const vscode = __importStar(require("vscode"));
const contextEngineClient_1 = require("./contextEngineClient");
const fileWatcher_1 = require("./fileWatcher");
const suggestionProvider_1 = require("./suggestionProvider");
const configurationManager_1 = require("./configurationManager");
const contextSuggestionsProvider_1 = require("./contextSuggestionsProvider");
const commands_1 = require("./commands");
const contextOverviewProvider_1 = require("./contextOverviewProvider");
const contextTreeProvider_1 = require("./contextTreeProvider");
const contextHealthProvider_1 = require("./contextHealthProvider");
const teamActivityProvider_1 = require("./teamActivityProvider");
const performanceMetricsProvider_1 = require("./performanceMetricsProvider");
const contextWizard_1 = require("./contextWizard");
let contextEngineClient;
let fileWatcher;
let suggestionProvider;
let configurationManager;
let contextSuggestionsProvider;
let commandManager;
let contextOverviewProvider;
let contextTreeProvider;
let contextHealthProvider;
let teamActivityProvider;
let performanceMetricsProvider;
let contextWizard;
function activate(context) {
    console.log('Professional Context Engine extension is now active!');
    // Initialize configuration manager
    configurationManager = new configurationManager_1.ConfigurationManager();
    // Initialize context engine client
    contextEngineClient = new contextEngineClient_1.ContextEngineClient(configurationManager);
    // Initialize file watcher
    fileWatcher = new fileWatcher_1.FileWatcher(contextEngineClient, configurationManager);
    // Initialize suggestion provider
    suggestionProvider = new suggestionProvider_1.SuggestionProvider(contextEngineClient, configurationManager);
    // Initialize context suggestions tree view provider
    contextSuggestionsProvider = new contextSuggestionsProvider_1.ContextSuggestionsProvider(contextEngineClient);
    // Initialize command manager
    commandManager = new commands_1.CommandManager(contextEngineClient, configurationManager);
    // Initialize professional-grade providers
    contextOverviewProvider = new contextOverviewProvider_1.ContextOverviewProvider(contextEngineClient);
    contextTreeProvider = new contextTreeProvider_1.ContextTreeProvider(contextEngineClient);
    contextHealthProvider = new contextHealthProvider_1.ContextHealthProvider(contextEngineClient);
    teamActivityProvider = new teamActivityProvider_1.TeamActivityProvider(contextEngineClient);
    performanceMetricsProvider = new performanceMetricsProvider_1.PerformanceMetricsProvider(contextEngineClient);
    contextWizard = new contextWizard_1.ContextWizard(contextEngineClient);
    // Register commands
    registerCommands(context);
    // Register additional commands from command manager
    commandManager.registerCommands(context);
    // Register providers
    registerProviders(context);
    // Start file watching
    fileWatcher.startWatching();
    // Connect to context engine server
    contextEngineClient.connect().catch(error => {
        vscode.window.showErrorMessage(`Failed to connect to Context Engine: ${error.message}`);
    });
    // Set context for when extension is enabled
    vscode.commands.executeCommand('setContext', 'contextEngine.enabled', true);
}
exports.activate = activate;
function registerCommands(context) {
    // Show context suggestions command
    const showSuggestionsCommand = vscode.commands.registerCommand('contextEngine.showSuggestions', async () => {
        const activeEditor = vscode.window.activeTextEditor;
        if (!activeEditor) {
            vscode.window.showInformationMessage('No active editor found');
            return;
        }
        try {
            const suggestions = await contextEngineClient.getSuggestions(activeEditor.document.uri.fsPath);
            if (suggestions.length === 0) {
                vscode.window.showInformationMessage('No context suggestions available for this file');
                return;
            }
            // Show suggestions in a quick pick
            const items = suggestions.map(suggestion => ({
                label: suggestion.title,
                description: suggestion.description,
                detail: `Priority: ${suggestion.priority}`,
                suggestion: suggestion
            }));
            const selected = await vscode.window.showQuickPick(items, {
                placeHolder: 'Select a context suggestion to apply'
            });
            if (selected) {
                await handleSuggestionAction(selected.suggestion);
            }
        }
        catch (error) {
            vscode.window.showErrorMessage(`Failed to get suggestions: ${error}`);
        }
    });
    // Refresh context analysis command
    const refreshContextCommand = vscode.commands.registerCommand('contextEngine.refreshContext', async () => {
        const activeEditor = vscode.window.activeTextEditor;
        if (!activeEditor) {
            vscode.window.showInformationMessage('No active editor found');
            return;
        }
        try {
            await contextEngineClient.analyzeFile(activeEditor.document.uri.fsPath);
            vscode.window.showInformationMessage('Context analysis refreshed');
            // Refresh the suggestions tree view
            contextSuggestionsProvider.refresh();
        }
        catch (error) {
            vscode.window.showErrorMessage(`Failed to refresh context: ${error}`);
        }
    });
    // Open settings command
    const openSettingsCommand = vscode.commands.registerCommand('contextEngine.openSettings', () => {
        vscode.commands.executeCommand('workbench.action.openSettings', 'contextEngine');
    });
    // Create context entry command
    const createContextCommand = vscode.commands.registerCommand('contextEngine.createContext', async () => {
        const activeEditor = vscode.window.activeTextEditor;
        if (!activeEditor) {
            vscode.window.showInformationMessage('No active editor found');
            return;
        }
        const selection = activeEditor.selection;
        const selectedText = activeEditor.document.getText(selection);
        // Get context details from user
        const title = await vscode.window.showInputBox({
            prompt: 'Enter context title',
            placeHolder: 'e.g., Business Rule: User Authentication'
        });
        if (!title) {
            return;
        }
        const contextType = await vscode.window.showQuickPick([
            'business_rule',
            'architectural_decision',
            'performance_requirement',
            'security_policy',
            'api_specification',
            'data_model',
            'workflow',
            'integration_point'
        ], {
            placeHolder: 'Select context type'
        });
        if (!contextType) {
            return;
        }
        const description = await vscode.window.showInputBox({
            prompt: 'Enter context description',
            placeHolder: 'Detailed description of the context...'
        });
        if (!description) {
            return;
        }
        try {
            await contextEngineClient.createContext({
                title,
                contextType,
                content: selectedText || description,
                description,
                filePath: activeEditor.document.uri.fsPath,
                lineNumber: selection.start.line + 1
            });
            vscode.window.showInformationMessage('Context entry created successfully');
        }
        catch (error) {
            vscode.window.showErrorMessage(`Failed to create context: ${error}`);
        }
    });
    // Analyze current file command
    const analyzeFileCommand = vscode.commands.registerCommand('contextEngine.analyzeFile', async () => {
        const activeEditor = vscode.window.activeTextEditor;
        if (!activeEditor) {
            vscode.window.showInformationMessage('No active editor found');
            return;
        }
        try {
            await vscode.window.withProgress({
                location: vscode.ProgressLocation.Notification,
                title: 'Analyzing file...',
                cancellable: false
            }, async () => {
                await contextEngineClient.analyzeFile(activeEditor.document.uri.fsPath);
            });
            vscode.window.showInformationMessage('File analysis completed');
            contextSuggestionsProvider.refresh();
        }
        catch (error) {
            vscode.window.showErrorMessage(`Failed to analyze file: ${error}`);
        }
    });
    // Context Wizard command
    const openContextWizardCommand = vscode.commands.registerCommand('contextEngine.openContextWizard', async () => {
        await contextWizard.showWizard();
    });
    // Context Tree commands
    const filterContextTreeCommand = vscode.commands.registerCommand('contextEngine.filterContextTree', async () => {
        await contextTreeProvider.showFilterDialog();
    });
    const searchContextTreeCommand = vscode.commands.registerCommand('contextEngine.searchContextTree', async () => {
        await contextTreeProvider.showSearchDialog();
    });
    const showRelationshipGraphCommand = vscode.commands.registerCommand('contextEngine.showRelationshipGraph', async () => {
        await contextTreeProvider.showGroupByDialog();
    });
    // Team Activity commands
    const showTeamActivityCommand = vscode.commands.registerCommand('contextEngine.showTeamActivity', async () => {
        teamActivityProvider.refresh();
    });
    const toggleSyncIndicatorCommand = vscode.commands.registerCommand('contextEngine.toggleSyncIndicator', async () => {
        const config = vscode.workspace.getConfiguration('contextEngine');
        const current = config.get('showSyncIndicators', true);
        await config.update('showSyncIndicators', !current, vscode.ConfigurationTarget.Workspace);
        vscode.window.showInformationMessage(`Sync indicators ${!current ? 'enabled' : 'disabled'}`);
    });
    // Performance Metrics commands
    const showPerformanceMetricsCommand = vscode.commands.registerCommand('contextEngine.showPerformanceMetrics', async () => {
        performanceMetricsProvider.refresh();
    });
    // Context details command
    const showContextDetailsCommand = vscode.commands.registerCommand('contextEngine.showContextDetails', async (contextItem) => {
        const panel = vscode.window.createWebviewPanel('contextDetails', `Context: ${contextItem.title}`, vscode.ViewColumn.Beside, { enableScripts: true });
        panel.webview.html = getContextDetailsWebview(contextItem);
    });
    // Register all commands
    context.subscriptions.push(showSuggestionsCommand, refreshContextCommand, openSettingsCommand, createContextCommand, analyzeFileCommand, openContextWizardCommand, filterContextTreeCommand, searchContextTreeCommand, showRelationshipGraphCommand, showTeamActivityCommand, toggleSyncIndicatorCommand, showPerformanceMetricsCommand, showContextDetailsCommand);
}
function registerProviders(context) {
    // Register hover provider
    const hoverProvider = vscode.languages.registerHoverProvider({ scheme: 'file' }, suggestionProvider);
    // Register code action provider
    const codeActionProvider = vscode.languages.registerCodeActionsProvider({ scheme: 'file' }, suggestionProvider, {
        providedCodeActionKinds: [vscode.CodeActionKind.QuickFix, vscode.CodeActionKind.Refactor]
    });
    // Register completion provider for intelligent suggestions
    const completionProvider = vscode.languages.registerCompletionItemProvider({ scheme: 'file' }, suggestionProvider, '.' // Trigger on dot
    );
    // Register tree data providers for activity bar views
    const contextOverviewTree = vscode.window.createTreeView('contextOverview', {
        treeDataProvider: contextOverviewProvider,
        showCollapseAll: false
    });
    const contextTreeView = vscode.window.createTreeView('contextTree', {
        treeDataProvider: contextTreeProvider,
        showCollapseAll: true
    });
    const contextSuggestionsTree = vscode.window.createTreeView('contextSuggestions', {
        treeDataProvider: contextSuggestionsProvider,
        showCollapseAll: true
    });
    const contextHealthTree = vscode.window.createTreeView('contextHealth', {
        treeDataProvider: contextHealthProvider,
        showCollapseAll: true
    });
    const teamActivityTree = vscode.window.createTreeView('teamActivity', {
        treeDataProvider: teamActivityProvider,
        showCollapseAll: true
    });
    const performanceMetricsTree = vscode.window.createTreeView('performanceMetrics', {
        treeDataProvider: performanceMetricsProvider,
        showCollapseAll: true
    });
    context.subscriptions.push(hoverProvider, codeActionProvider, completionProvider, contextOverviewTree, contextTreeView, contextSuggestionsTree, contextHealthTree, teamActivityTree, performanceMetricsTree);
}
async function handleSuggestionAction(suggestion) {
    if (!suggestion.actions || suggestion.actions.length === 0) {
        vscode.window.showInformationMessage('No actions available for this suggestion');
        return;
    }
    // If only one action, execute it directly
    if (suggestion.actions.length === 1) {
        await executeSuggestionAction(suggestion.actions[0], suggestion);
        return;
    }
    // Multiple actions - let user choose
    const actionItems = suggestion.actions.map((action) => ({
        label: action.title,
        description: action.description,
        action: action
    }));
    const selectedAction = await vscode.window.showQuickPick(actionItems, {
        placeHolder: 'Select an action to perform'
    });
    if (selectedAction) {
        await executeSuggestionAction(selectedAction.action, suggestion);
    }
}
async function executeSuggestionAction(action, suggestion) {
    try {
        switch (action.action_type) {
            case 'CreateContext':
                await vscode.commands.executeCommand('contextEngine.createContext');
                break;
            case 'UpdateContext':
                // Handle context update
                vscode.window.showInformationMessage('Context update functionality not yet implemented');
                break;
            case 'NavigateToCode':
                if (suggestion.file_path && suggestion.line_number) {
                    const document = await vscode.workspace.openTextDocument(suggestion.file_path);
                    const editor = await vscode.window.showTextDocument(document);
                    const position = new vscode.Position(suggestion.line_number - 1, 0);
                    editor.selection = new vscode.Selection(position, position);
                    editor.revealRange(new vscode.Range(position, position));
                }
                break;
            case 'ShowDocumentation':
                // Show documentation in a webview or external browser
                vscode.window.showInformationMessage('Documentation display functionality not yet implemented');
                break;
            case 'RunAnalysis':
                await vscode.commands.executeCommand('contextEngine.analyzeFile');
                break;
            case 'ApplyFix':
                // Apply automated fix
                vscode.window.showInformationMessage('Automated fix functionality not yet implemented');
                break;
            default:
                vscode.window.showWarningMessage(`Unknown action type: ${action.action_type}`);
        }
    }
    catch (error) {
        vscode.window.showErrorMessage(`Failed to execute action: ${error}`);
    }
}
function getContextDetailsWebview(contextItem) {
    return `
        <!DOCTYPE html>
        <html>
        <head>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>Context Details</title>
            <style>
                body {
                    font-family: var(--vscode-font-family);
                    padding: 20px;
                    color: var(--vscode-foreground);
                    background-color: var(--vscode-editor-background);
                }
                .header {
                    border-bottom: 1px solid var(--vscode-panel-border);
                    padding-bottom: 15px;
                    margin-bottom: 20px;
                }
                .metadata {
                    background-color: var(--vscode-editor-inactiveSelectionBackground);
                    padding: 15px;
                    border-radius: 4px;
                    margin: 15px 0;
                }
                .quality-score {
                    display: inline-block;
                    padding: 4px 8px;
                    border-radius: 4px;
                    font-weight: bold;
                    color: white;
                }
                .quality-high { background-color: #28a745; }
                .quality-medium { background-color: #ffc107; color: black; }
                .quality-low { background-color: #dc3545; }
                .tags {
                    margin-top: 10px;
                }
                .tag {
                    display: inline-block;
                    background-color: var(--vscode-button-background);
                    color: var(--vscode-button-foreground);
                    padding: 2px 6px;
                    border-radius: 3px;
                    font-size: 12px;
                    margin-right: 5px;
                }
                .relationships {
                    margin-top: 20px;
                }
                .relationship {
                    padding: 8px;
                    margin: 5px 0;
                    background-color: var(--vscode-editor-inactiveSelectionBackground);
                    border-left: 3px solid var(--vscode-textLink-foreground);
                    border-radius: 0 4px 4px 0;
                }
            </style>
        </head>
        <body>
            <div class="header">
                <h1>${contextItem.title}</h1>
                <p><strong>Type:</strong> ${contextItem.type}</p>
                ${contextItem.qualityScore ? `
                    <span class="quality-score quality-${contextItem.qualityScore > 0.7 ? 'high' : contextItem.qualityScore > 0.4 ? 'medium' : 'low'}">
                        Quality: ${Math.round(contextItem.qualityScore * 100)}%
                    </span>
                ` : ''}
            </div>
            
            <div class="content">
                <h2>Description</h2>
                <p>${contextItem.description}</p>
                
                <div class="metadata">
                    ${contextItem.filePath ? `<p><strong>File:</strong> ${contextItem.filePath}</p>` : ''}
                    ${contextItem.lineNumber ? `<p><strong>Line:</strong> ${contextItem.lineNumber}</p>` : ''}
                    ${contextItem.lastModified ? `<p><strong>Last Modified:</strong> ${new Date(contextItem.lastModified).toLocaleString()}</p>` : ''}
                </div>
                
                ${contextItem.tags && contextItem.tags.length > 0 ? `
                    <div class="tags">
                        <h3>Tags</h3>
                        ${contextItem.tags.map((tag) => `<span class="tag">${tag}</span>`).join('')}
                    </div>
                ` : ''}
                
                ${contextItem.relationships && contextItem.relationships.length > 0 ? `
                    <div class="relationships">
                        <h3>Relationships</h3>
                        ${contextItem.relationships.map((rel) => `
                            <div class="relationship">
                                <strong>${rel.type}:</strong> ${rel.targetId} (${Math.round(rel.strength * 100)}% strength)
                            </div>
                        `).join('')}
                    </div>
                ` : ''}
            </div>
        </body>
        </html>
    `;
}
function deactivate() {
    if (fileWatcher) {
        fileWatcher.dispose();
    }
    if (contextEngineClient) {
        contextEngineClient.disconnect();
    }
    if (contextHealthProvider) {
        contextHealthProvider.dispose();
    }
    if (performanceMetricsProvider) {
        performanceMetricsProvider.dispose();
    }
}
exports.deactivate = deactivate;
//# sourceMappingURL=extension.js.map