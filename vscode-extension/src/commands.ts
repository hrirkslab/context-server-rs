import * as vscode from 'vscode';
import { ContextEngineClient, ContextSuggestion } from './contextEngineClient';
import { ConfigurationManager, AnalysisRule, SuggestionTrigger } from './configurationManager';

export class CommandManager {
    constructor(
        private contextClient: ContextEngineClient,
        private configManager: ConfigurationManager
    ) { }

    registerCommands(context: vscode.ExtensionContext): void {
        // Register all commands
        const commands = [
            vscode.commands.registerCommand('contextEngine.executeSuggestionAction', this.executeSuggestionAction.bind(this)),
            vscode.commands.registerCommand('contextEngine.showSuggestionDetails', this.showSuggestionDetails.bind(this)),
            vscode.commands.registerCommand('contextEngine.configureAnalysisRules', this.configureAnalysisRules.bind(this)),
            vscode.commands.registerCommand('contextEngine.configureSuggestionTriggers', this.configureSuggestionTriggers.bind(this)),
            vscode.commands.registerCommand('contextEngine.exportConfiguration', this.exportConfiguration.bind(this)),
            vscode.commands.registerCommand('contextEngine.importConfiguration', this.importConfiguration.bind(this)),
            vscode.commands.registerCommand('contextEngine.showContextHealth', this.showContextHealth.bind(this)),
            vscode.commands.registerCommand('contextEngine.showProjectInsights', this.showProjectInsights.bind(this)),
            vscode.commands.registerCommand('contextEngine.queryContext', this.queryContext.bind(this)),
            vscode.commands.registerCommand('contextEngine.toggleRealTimeSuggestions', this.toggleRealTimeSuggestions.bind(this)),
            vscode.commands.registerCommand('contextEngine.toggleAutoAnalyze', this.toggleAutoAnalyze.bind(this)),
            vscode.commands.registerCommand('contextEngine.addAnalysisRule', this.addAnalysisRule.bind(this)),
            vscode.commands.registerCommand('contextEngine.addSuggestionTrigger', this.addSuggestionTrigger.bind(this)),
            vscode.commands.registerCommand('contextEngine.testConnection', this.testConnection.bind(this))
        ];

        context.subscriptions.push(...commands);
    }

    private async executeSuggestionAction(params: any): Promise<void> {
        try {
            const { suggestionId, actionType, suggestion, action } = params;

            switch (actionType) {
                case 'CreateContext':
                    await this.handleCreateContextAction(suggestion, action);
                    break;
                case 'UpdateContext':
                    await this.handleUpdateContextAction(suggestion, action);
                    break;
                case 'NavigateToCode':
                    await this.handleNavigateToCodeAction(suggestion, action);
                    break;
                case 'ShowDocumentation':
                    await this.handleShowDocumentationAction(suggestion, action);
                    break;
                case 'RunAnalysis':
                    await this.handleRunAnalysisAction(suggestion, action);
                    break;
                case 'ApplyFix':
                    await this.handleApplyFixAction(suggestion, action);
                    break;
                default:
                    vscode.window.showWarningMessage(`Unknown action type: ${actionType}`);
            }
        } catch (error) {
            vscode.window.showErrorMessage(`Failed to execute action: ${error}`);
        }
    }

    private async handleCreateContextAction(suggestion: ContextSuggestion, action: any): Promise<void> {
        const activeEditor = vscode.window.activeTextEditor;
        if (!activeEditor) {
            vscode.window.showInformationMessage('No active editor found');
            return;
        }

        const selection = activeEditor.selection;
        const selectedText = activeEditor.document.getText(selection);

        // Pre-fill with suggestion data
        const title = await vscode.window.showInputBox({
            prompt: 'Enter context title',
            value: suggestion.title,
            placeHolder: 'e.g., Business Rule: User Authentication'
        });

        if (!title) return;

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

        if (!contextType) return;

        const description = await vscode.window.showInputBox({
            prompt: 'Enter context description',
            value: suggestion.description,
            placeHolder: 'Detailed description of the context...'
        });

        if (!description) return;

        await this.contextClient.createContext({
            title,
            contextType,
            content: selectedText || description,
            description,
            filePath: activeEditor.document.uri.fsPath,
            lineNumber: selection.start.line + 1
        });

        vscode.window.showInformationMessage('Context entry created successfully');
    }

    private async handleUpdateContextAction(suggestion: ContextSuggestion, action: any): Promise<void> {
        vscode.window.showInformationMessage('Context update functionality will be implemented in a future version');
    }

    private async handleNavigateToCodeAction(suggestion: ContextSuggestion, action: any): Promise<void> {
        if (suggestion.file_path && suggestion.line_number) {
            try {
                const document = await vscode.workspace.openTextDocument(suggestion.file_path);
                const editor = await vscode.window.showTextDocument(document);
                const position = new vscode.Position(suggestion.line_number - 1, 0);
                editor.selection = new vscode.Selection(position, position);
                editor.revealRange(new vscode.Range(position, position));
            } catch (error) {
                vscode.window.showErrorMessage(`Failed to navigate to code: ${error}`);
            }
        } else {
            vscode.window.showWarningMessage('No file location available for this suggestion');
        }
    }

    private async handleShowDocumentationAction(suggestion: ContextSuggestion, action: any): Promise<void> {
        // Create a webview to show documentation
        const panel = vscode.window.createWebviewPanel(
            'contextDocumentation',
            `Documentation: ${suggestion.title}`,
            vscode.ViewColumn.Beside,
            {
                enableScripts: true,
                retainContextWhenHidden: true
            }
        );

        panel.webview.html = this.getDocumentationWebviewContent(suggestion);
    }

    private async handleRunAnalysisAction(suggestion: ContextSuggestion, action: any): Promise<void> {
        const activeEditor = vscode.window.activeTextEditor;
        if (!activeEditor) {
            vscode.window.showInformationMessage('No active editor found');
            return;
        }

        await vscode.window.withProgress({
            location: vscode.ProgressLocation.Notification,
            title: 'Running context analysis...',
            cancellable: false
        }, async () => {
            await this.contextClient.analyzeFile(activeEditor.document.uri.fsPath);
        });

        vscode.window.showInformationMessage('Analysis completed');
    }

    private async handleApplyFixAction(suggestion: ContextSuggestion, action: any): Promise<void> {
        vscode.window.showInformationMessage('Automated fix functionality will be implemented in a future version');
    }

    private async showSuggestionDetails(suggestion: ContextSuggestion): Promise<void> {
        const panel = vscode.window.createWebviewPanel(
            'suggestionDetails',
            `Suggestion: ${suggestion.title}`,
            vscode.ViewColumn.Beside,
            {
                enableScripts: true,
                retainContextWhenHidden: true
            }
        );

        panel.webview.html = this.getSuggestionDetailsWebviewContent(suggestion);
    }

    private async configureAnalysisRules(): Promise<void> {
        const action = await vscode.window.showQuickPick([
            'Add New Rule',
            'View Existing Rules',
            'Remove Rule'
        ], {
            placeHolder: 'Select an action'
        });

        switch (action) {
            case 'Add New Rule':
                await this.addAnalysisRule();
                break;
            case 'View Existing Rules':
                await this.viewAnalysisRules();
                break;
            case 'Remove Rule':
                await this.removeAnalysisRule();
                break;
        }
    }

    private async addAnalysisRule(): Promise<void> {
        const name = await vscode.window.showInputBox({
            prompt: 'Enter rule name',
            placeHolder: 'e.g., Extract TODO Comments'
        });
        if (!name) return;

        const language = await vscode.window.showQuickPick(
            this.configManager.getSupportedLanguages(),
            { placeHolder: 'Select target language' }
        );
        if (!language) return;

        const pattern = await vscode.window.showInputBox({
            prompt: 'Enter regex pattern',
            placeHolder: 'e.g., TODO:\\s*(.+)'
        });
        if (!pattern) return;

        const contextType = await vscode.window.showQuickPick([
            'business_rule',
            'architectural_decision',
            'performance_requirement',
            'security_policy',
            'api_specification',
            'data_model',
            'workflow',
            'integration_point',
            'general'
        ], {
            placeHolder: 'Select context type'
        });
        if (!contextType) return;

        const extractionMethod = await vscode.window.showQuickPick([
            'RegexCapture',
            'LineContent',
            'BlockContent',
            'Custom'
        ], {
            placeHolder: 'Select extraction method'
        });
        if (!extractionMethod) return;

        const confidenceStr = await vscode.window.showInputBox({
            prompt: 'Enter confidence level (0.0 - 1.0)',
            value: '0.8',
            validateInput: (value) => {
                const num = parseFloat(value);
                if (isNaN(num) || num < 0 || num > 1) {
                    return 'Confidence must be a number between 0.0 and 1.0';
                }
                return null;
            }
        });
        if (!confidenceStr) return;

        const rule: AnalysisRule = {
            name,
            language,
            pattern,
            contextType,
            extractionMethod,
            confidence: parseFloat(confidenceStr)
        };

        const errors = this.configManager.validateAnalysisRule(rule);
        if (errors.length > 0) {
            vscode.window.showErrorMessage(`Validation errors: ${errors.join(', ')}`);
            return;
        }

        await this.configManager.addAnalysisRule(rule);
        vscode.window.showInformationMessage('Analysis rule added successfully');
    }

    private async viewAnalysisRules(): Promise<void> {
        const rules = this.configManager.getAnalysisRules();
        if (rules.length === 0) {
            vscode.window.showInformationMessage('No analysis rules configured');
            return;
        }

        const items = rules.map(rule => ({
            label: rule.name,
            description: `${rule.language} - ${rule.contextType}`,
            detail: `Pattern: ${rule.pattern} | Confidence: ${rule.confidence}`,
            rule
        }));

        const selected = await vscode.window.showQuickPick(items, {
            placeHolder: 'Select a rule to view details'
        });

        if (selected) {
            const panel = vscode.window.createWebviewPanel(
                'analysisRuleDetails',
                `Analysis Rule: ${selected.rule.name}`,
                vscode.ViewColumn.Beside,
                { enableScripts: false }
            );

            panel.webview.html = this.getAnalysisRuleWebviewContent(selected.rule);
        }
    }

    private async removeAnalysisRule(): Promise<void> {
        const rules = this.configManager.getAnalysisRules();
        if (rules.length === 0) {
            vscode.window.showInformationMessage('No analysis rules to remove');
            return;
        }

        const items = rules.map(rule => ({
            label: rule.name,
            description: `${rule.language} - ${rule.contextType}`,
            rule
        }));

        const selected = await vscode.window.showQuickPick(items, {
            placeHolder: 'Select a rule to remove'
        });

        if (selected) {
            const confirm = await vscode.window.showWarningMessage(
                `Are you sure you want to remove the rule "${selected.rule.name}"?`,
                'Yes', 'No'
            );

            if (confirm === 'Yes') {
                await this.configManager.removeAnalysisRule(selected.rule.name);
                vscode.window.showInformationMessage('Analysis rule removed successfully');
            }
        }
    }

    private async configureSuggestionTriggers(): Promise<void> {
        const action = await vscode.window.showQuickPick([
            'Add New Trigger',
            'View Existing Triggers',
            'Remove Trigger'
        ], {
            placeHolder: 'Select an action'
        });

        switch (action) {
            case 'Add New Trigger':
                await this.addSuggestionTrigger();
                break;
            case 'View Existing Triggers':
                await this.viewSuggestionTriggers();
                break;
            case 'Remove Trigger':
                await this.removeSuggestionTrigger();
                break;
        }
    }

    private async addSuggestionTrigger(): Promise<void> {
        const name = await vscode.window.showInputBox({
            prompt: 'Enter trigger name',
            placeHolder: 'e.g., Large Function Warning'
        });
        if (!name) return;

        const triggerType = await vscode.window.showQuickPick([
            'TextChange',
            'FileSave',
            'FileOpen',
            'LineCount',
            'ComplexityThreshold',
            'PatternMatch'
        ], {
            placeHolder: 'Select trigger type'
        });
        if (!triggerType) return;

        const condition = await vscode.window.showInputBox({
            prompt: 'Enter trigger condition',
            placeHolder: 'e.g., lineCount > 50'
        });
        if (!condition) return;

        const suggestionTemplate = await vscode.window.showInputBox({
            prompt: 'Enter suggestion template',
            placeHolder: 'e.g., Consider breaking this function into smaller parts'
        });
        if (!suggestionTemplate) return;

        const trigger: SuggestionTrigger = {
            name,
            triggerType,
            condition,
            suggestionTemplate
        };

        const errors = this.configManager.validateSuggestionTrigger(trigger);
        if (errors.length > 0) {
            vscode.window.showErrorMessage(`Validation errors: ${errors.join(', ')}`);
            return;
        }

        await this.configManager.addSuggestionTrigger(trigger);
        vscode.window.showInformationMessage('Suggestion trigger added successfully');
    }

    private async viewSuggestionTriggers(): Promise<void> {
        const triggers = this.configManager.getSuggestionTriggers();
        if (triggers.length === 0) {
            vscode.window.showInformationMessage('No suggestion triggers configured');
            return;
        }

        const items = triggers.map(trigger => ({
            label: trigger.name,
            description: trigger.triggerType,
            detail: `Condition: ${trigger.condition}`,
            trigger
        }));

        const selected = await vscode.window.showQuickPick(items, {
            placeHolder: 'Select a trigger to view details'
        });

        if (selected) {
            const panel = vscode.window.createWebviewPanel(
                'suggestionTriggerDetails',
                `Suggestion Trigger: ${selected.trigger.name}`,
                vscode.ViewColumn.Beside,
                { enableScripts: false }
            );

            panel.webview.html = this.getSuggestionTriggerWebviewContent(selected.trigger);
        }
    }

    private async removeSuggestionTrigger(): Promise<void> {
        const triggers = this.configManager.getSuggestionTriggers();
        if (triggers.length === 0) {
            vscode.window.showInformationMessage('No suggestion triggers to remove');
            return;
        }

        const items = triggers.map(trigger => ({
            label: trigger.name,
            description: trigger.triggerType,
            trigger
        }));

        const selected = await vscode.window.showQuickPick(items, {
            placeHolder: 'Select a trigger to remove'
        });

        if (selected) {
            const confirm = await vscode.window.showWarningMessage(
                `Are you sure you want to remove the trigger "${selected.trigger.name}"?`,
                'Yes', 'No'
            );

            if (confirm === 'Yes') {
                await this.configManager.removeSuggestionTrigger(selected.trigger.name);
                vscode.window.showInformationMessage('Suggestion trigger removed successfully');
            }
        }
    }

    private async exportConfiguration(): Promise<void> {
        const config = this.configManager.exportConfiguration();
        const configJson = JSON.stringify(config, null, 2);

        const uri = await vscode.window.showSaveDialog({
            defaultUri: vscode.Uri.file('context-engine-config.json'),
            filters: {
                'JSON Files': ['json']
            }
        });

        if (uri) {
            await vscode.workspace.fs.writeFile(uri, Buffer.from(configJson, 'utf8'));
            vscode.window.showInformationMessage('Configuration exported successfully');
        }
    }

    private async importConfiguration(): Promise<void> {
        const uri = await vscode.window.showOpenDialog({
            canSelectFiles: true,
            canSelectFolders: false,
            canSelectMany: false,
            filters: {
                'JSON Files': ['json']
            }
        });

        if (uri && uri.length > 0) {
            try {
                const content = await vscode.workspace.fs.readFile(uri[0]);
                const configData = JSON.parse(content.toString());

                await this.configManager.importConfiguration(configData);
                vscode.window.showInformationMessage('Configuration imported successfully');
            } catch (error) {
                vscode.window.showErrorMessage(`Failed to import configuration: ${error}`);
            }
        }
    }

    private async showContextHealth(): Promise<void> {
        try {
            const health = await this.contextClient.getContextHealth();

            const panel = vscode.window.createWebviewPanel(
                'contextHealth',
                'Context Health Report',
                vscode.ViewColumn.Beside,
                {
                    enableScripts: true,
                    retainContextWhenHidden: true
                }
            );

            panel.webview.html = this.getContextHealthWebviewContent(health);
        } catch (error) {
            vscode.window.showErrorMessage(`Failed to get context health: ${error}`);
        }
    }

    private async showProjectInsights(): Promise<void> {
        try {
            const insights = await this.contextClient.getProjectInsights();

            const panel = vscode.window.createWebviewPanel(
                'projectInsights',
                'Project Insights',
                vscode.ViewColumn.Beside,
                {
                    enableScripts: true,
                    retainContextWhenHidden: true
                }
            );

            panel.webview.html = this.getProjectInsightsWebviewContent(insights);
        } catch (error) {
            vscode.window.showErrorMessage(`Failed to get project insights: ${error}`);
        }
    }

    private async queryContext(): Promise<void> {
        const query = await vscode.window.showInputBox({
            prompt: 'Enter your context query',
            placeHolder: 'e.g., authentication business rules'
        });

        if (!query) return;

        try {
            const results = await this.contextClient.queryContext(query);

            if (results.length === 0) {
                vscode.window.showInformationMessage('No context found for your query');
                return;
            }

            const panel = vscode.window.createWebviewPanel(
                'contextQueryResults',
                `Query Results: ${query}`,
                vscode.ViewColumn.Beside,
                {
                    enableScripts: true,
                    retainContextWhenHidden: true
                }
            );

            panel.webview.html = this.getQueryResultsWebviewContent(query, results);
        } catch (error) {
            vscode.window.showErrorMessage(`Failed to query context: ${error}`);
        }
    }

    private async toggleRealTimeSuggestions(): Promise<void> {
        const current = this.configManager.isRealTimeSuggestionsEnabled();
        await this.configManager.updateRealTimeSuggestions(!current);

        const status = !current ? 'enabled' : 'disabled';
        vscode.window.showInformationMessage(`Real-time suggestions ${status}`);
    }

    private async toggleAutoAnalyze(): Promise<void> {
        const current = this.configManager.isAutoAnalyzeOnSaveEnabled();
        await this.configManager.updateAutoAnalyzeOnSave(!current);

        const status = !current ? 'enabled' : 'disabled';
        vscode.window.showInformationMessage(`Auto-analyze on save ${status}`);
    }

    private async testConnection(): Promise<void> {
        try {
            await vscode.window.withProgress({
                location: vscode.ProgressLocation.Notification,
                title: 'Testing connection to Context Engine...',
                cancellable: false
            }, async () => {
                await this.contextClient.connect();
            });

            const isConnected = this.contextClient.isConnected();
            if (isConnected) {
                vscode.window.showInformationMessage('Successfully connected to Context Engine');
            } else {
                vscode.window.showWarningMessage('Connected to HTTP endpoint but WebSocket connection failed');
            }
        } catch (error) {
            vscode.window.showErrorMessage(`Connection test failed: ${error}`);
        }
    }

    // Webview content generators
    private getDocumentationWebviewContent(suggestion: ContextSuggestion): string {
        return `
            <!DOCTYPE html>
            <html>
            <head>
                <meta charset="UTF-8">
                <meta name="viewport" content="width=device-width, initial-scale=1.0">
                <title>Documentation</title>
                <style>
                    body { font-family: var(--vscode-font-family); padding: 20px; }
                    .header { border-bottom: 1px solid var(--vscode-panel-border); padding-bottom: 10px; margin-bottom: 20px; }
                    .priority { padding: 4px 8px; border-radius: 4px; font-size: 12px; font-weight: bold; }
                    .priority.high { background-color: var(--vscode-errorForeground); color: white; }
                    .priority.medium { background-color: var(--vscode-warningForeground); color: white; }
                    .priority.low { background-color: var(--vscode-infoForeground); color: white; }
                    .metadata { background-color: var(--vscode-editor-background); padding: 10px; border-radius: 4px; margin: 10px 0; }
                </style>
            </head>
            <body>
                <div class="header">
                    <h1>${suggestion.title}</h1>
                    <span class="priority ${suggestion.priority.toLowerCase()}">${suggestion.priority}</span>
                </div>
                <div class="content">
                    <h2>Description</h2>
                    <p>${suggestion.description}</p>
                    
                    <h2>Type</h2>
                    <p>${suggestion.suggestion_type}</p>
                    
                    ${suggestion.file_path ? `
                        <h2>Location</h2>
                        <div class="metadata">
                            <strong>File:</strong> ${suggestion.file_path}<br>
                            ${suggestion.line_number ? `<strong>Line:</strong> ${suggestion.line_number}` : ''}
                        </div>
                    ` : ''}
                    
                    ${Object.keys(suggestion.metadata).length > 0 ? `
                        <h2>Additional Information</h2>
                        <div class="metadata">
                            ${Object.entries(suggestion.metadata).map(([key, value]) =>
            `<strong>${key}:</strong> ${value}<br>`
        ).join('')}
                        </div>
                    ` : ''}
                </div>
            </body>
            </html>
        `;
    }

    private getSuggestionDetailsWebviewContent(suggestion: ContextSuggestion): string {
        return `
            <!DOCTYPE html>
            <html>
            <head>
                <meta charset="UTF-8">
                <meta name="viewport" content="width=device-width, initial-scale=1.0">
                <title>Suggestion Details</title>
                <style>
                    body { font-family: var(--vscode-font-family); padding: 20px; }
                    .header { border-bottom: 1px solid var(--vscode-panel-border); padding-bottom: 10px; margin-bottom: 20px; }
                    .priority { padding: 4px 8px; border-radius: 4px; font-size: 12px; font-weight: bold; }
                    .priority.critical { background-color: #ff0000; color: white; }
                    .priority.high { background-color: #ff8800; color: white; }
                    .priority.medium { background-color: #ffaa00; color: white; }
                    .priority.low { background-color: #888888; color: white; }
                    .action { background-color: var(--vscode-button-background); color: var(--vscode-button-foreground); padding: 8px 12px; margin: 5px; border: none; border-radius: 4px; cursor: pointer; }
                    .action:hover { background-color: var(--vscode-button-hoverBackground); }
                    .metadata { background-color: var(--vscode-editor-background); padding: 10px; border-radius: 4px; margin: 10px 0; }
                </style>
            </head>
            <body>
                <div class="header">
                    <h1>${suggestion.title}</h1>
                    <span class="priority ${suggestion.priority.toLowerCase()}">${suggestion.priority} Priority</span>
                </div>
                
                <div class="content">
                    <h2>Description</h2>
                    <p>${suggestion.description}</p>
                    
                    <h2>Suggestion Type</h2>
                    <p>${suggestion.suggestion_type}</p>
                    
                    ${suggestion.file_path ? `
                        <h2>Location</h2>
                        <div class="metadata">
                            <strong>File:</strong> ${suggestion.file_path}<br>
                            ${suggestion.line_number ? `<strong>Line:</strong> ${suggestion.line_number}` : ''}
                        </div>
                    ` : ''}
                    
                    <h2>Available Actions</h2>
                    <div class="actions">
                        ${suggestion.actions.map(action => `
                            <div class="action-item">
                                <h3>${action.title}</h3>
                                <p>${action.description}</p>
                                <button class="action" onclick="executeAction('${action.action_type}')">
                                    ${action.title}
                                </button>
                            </div>
                        `).join('')}
                    </div>
                    
                    ${Object.keys(suggestion.metadata).length > 0 ? `
                        <h2>Metadata</h2>
                        <div class="metadata">
                            ${Object.entries(suggestion.metadata).map(([key, value]) =>
            `<strong>${key}:</strong> ${JSON.stringify(value)}<br>`
        ).join('')}
                        </div>
                    ` : ''}
                </div>
                
                <script>
                    function executeAction(actionType) {
                        // This would communicate back to the extension
                        console.log('Execute action:', actionType);
                    }
                </script>
            </body>
            </html>
        `;
    }

    private getAnalysisRuleWebviewContent(rule: AnalysisRule): string {
        return `
            <!DOCTYPE html>
            <html>
            <head>
                <meta charset="UTF-8">
                <meta name="viewport" content="width=device-width, initial-scale=1.0">
                <title>Analysis Rule Details</title>
                <style>
                    body { font-family: var(--vscode-font-family); padding: 20px; }
                    .rule-detail { margin: 10px 0; }
                    .rule-detail strong { color: var(--vscode-textLink-foreground); }
                    .code { background-color: var(--vscode-editor-background); padding: 10px; border-radius: 4px; font-family: monospace; }
                </style>
            </head>
            <body>
                <h1>Analysis Rule: ${rule.name}</h1>
                
                <div class="rule-detail">
                    <strong>Language:</strong> ${rule.language}
                </div>
                
                <div class="rule-detail">
                    <strong>Context Type:</strong> ${rule.contextType}
                </div>
                
                <div class="rule-detail">
                    <strong>Extraction Method:</strong> ${rule.extractionMethod}
                </div>
                
                <div class="rule-detail">
                    <strong>Confidence:</strong> ${rule.confidence}
                </div>
                
                <div class="rule-detail">
                    <strong>Pattern:</strong>
                    <div class="code">${rule.pattern}</div>
                </div>
            </body>
            </html>
        `;
    }

    private getSuggestionTriggerWebviewContent(trigger: SuggestionTrigger): string {
        return `
            <!DOCTYPE html>
            <html>
            <head>
                <meta charset="UTF-8">
                <meta name="viewport" content="width=device-width, initial-scale=1.0">
                <title>Suggestion Trigger Details</title>
                <style>
                    body { font-family: var(--vscode-font-family); padding: 20px; }
                    .trigger-detail { margin: 10px 0; }
                    .trigger-detail strong { color: var(--vscode-textLink-foreground); }
                    .code { background-color: var(--vscode-editor-background); padding: 10px; border-radius: 4px; font-family: monospace; }
                </style>
            </head>
            <body>
                <h1>Suggestion Trigger: ${trigger.name}</h1>
                
                <div class="trigger-detail">
                    <strong>Trigger Type:</strong> ${trigger.triggerType}
                </div>
                
                <div class="trigger-detail">
                    <strong>Condition:</strong>
                    <div class="code">${trigger.condition}</div>
                </div>
                
                <div class="trigger-detail">
                    <strong>Suggestion Template:</strong>
                    <div class="code">${trigger.suggestionTemplate}</div>
                </div>
            </body>
            </html>
        `;
    }

    private getContextHealthWebviewContent(health: any): string {
        return `
            <!DOCTYPE html>
            <html>
            <head>
                <meta charset="UTF-8">
                <meta name="viewport" content="width=device-width, initial-scale=1.0">
                <title>Context Health Report</title>
                <style>
                    body { font-family: var(--vscode-font-family); padding: 20px; }
                    .health-metric { margin: 15px 0; padding: 10px; border-radius: 4px; background-color: var(--vscode-editor-background); }
                    .health-score { font-size: 24px; font-weight: bold; }
                    .health-good { color: #4CAF50; }
                    .health-warning { color: #FF9800; }
                    .health-error { color: #F44336; }
                </style>
            </head>
            <body>
                <h1>Context Health Report</h1>
                
                <div class="health-metric">
                    <h2>Overall Health Score</h2>
                    <div class="health-score ${this.getHealthClass(health.overall_score || 0)}">
                        ${Math.round((health.overall_score || 0) * 100)}%
                    </div>
                </div>
                
                <div class="health-metric">
                    <h2>Context Statistics</h2>
                    <p><strong>Total Context Items:</strong> ${health.total_contexts || 0}</p>
                    <p><strong>Active Contexts:</strong> ${health.active_contexts || 0}</p>
                    <p><strong>Stale Contexts:</strong> ${health.stale_contexts || 0}</p>
                </div>
                
                <div class="health-metric">
                    <h2>Quality Metrics</h2>
                    <p><strong>Average Quality Score:</strong> ${Math.round((health.average_quality || 0) * 100)}%</p>
                    <p><strong>Contexts Needing Review:</strong> ${health.needs_review || 0}</p>
                </div>
                
                ${health.recommendations && health.recommendations.length > 0 ? `
                    <div class="health-metric">
                        <h2>Recommendations</h2>
                        <ul>
                            ${health.recommendations.map((rec: string) => `<li>${rec}</li>`).join('')}
                        </ul>
                    </div>
                ` : ''}
            </body>
            </html>
        `;
    }

    private getProjectInsightsWebviewContent(insights: any): string {
        return `
            <!DOCTYPE html>
            <html>
            <head>
                <meta charset="UTF-8">
                <meta name="viewport" content="width=device-width, initial-scale=1.0">
                <title>Project Insights</title>
                <style>
                    body { font-family: var(--vscode-font-family); padding: 20px; }
                    .insight-section { margin: 20px 0; padding: 15px; border-radius: 4px; background-color: var(--vscode-editor-background); }
                    .metric { display: inline-block; margin: 10px; padding: 10px; background-color: var(--vscode-button-background); border-radius: 4px; }
                    .chart-placeholder { height: 200px; background-color: var(--vscode-input-background); border-radius: 4px; display: flex; align-items: center; justify-content: center; margin: 10px 0; }
                </style>
            </head>
            <body>
                <h1>Project Insights</h1>
                
                <div class="insight-section">
                    <h2>Context Usage</h2>
                    <div class="metric">
                        <strong>Total Queries:</strong> ${insights.total_queries || 0}
                    </div>
                    <div class="metric">
                        <strong>Successful Queries:</strong> ${insights.successful_queries || 0}
                    </div>
                    <div class="metric">
                        <strong>Success Rate:</strong> ${Math.round((insights.success_rate || 0) * 100)}%
                    </div>
                </div>
                
                <div class="insight-section">
                    <h2>Most Used Context Types</h2>
                    ${insights.popular_context_types ?
                insights.popular_context_types.map((type: any) => `
                            <div class="metric">
                                <strong>${type.name}:</strong> ${type.count} uses
                            </div>
                        `).join('') :
                '<p>No usage data available</p>'
            }
                </div>
                
                <div class="insight-section">
                    <h2>Development Velocity</h2>
                    <div class="chart-placeholder">
                        Context usage trends would be displayed here
                    </div>
                </div>
                
                ${insights.recommendations && insights.recommendations.length > 0 ? `
                    <div class="insight-section">
                        <h2>Recommendations</h2>
                        <ul>
                            ${insights.recommendations.map((rec: string) => `<li>${rec}</li>`).join('')}
                        </ul>
                    </div>
                ` : ''}
            </body>
            </html>
        `;
    }

    private getQueryResultsWebviewContent(query: string, results: any[]): string {
        return `
            <!DOCTYPE html>
            <html>
            <head>
                <meta charset="UTF-8">
                <meta name="viewport" content="width=device-width, initial-scale=1.0">
                <title>Query Results</title>
                <style>
                    body { font-family: var(--vscode-font-family); padding: 20px; }
                    .result-item { margin: 15px 0; padding: 15px; border-radius: 4px; background-color: var(--vscode-editor-background); border-left: 4px solid var(--vscode-textLink-foreground); }
                    .result-title { font-weight: bold; margin-bottom: 5px; }
                    .result-meta { font-size: 12px; color: var(--vscode-descriptionForeground); margin-top: 10px; }
                </style>
            </head>
            <body>
                <h1>Query Results for: "${query}"</h1>
                <p>Found ${results.length} result(s)</p>
                
                ${results.map((result, index) => `
                    <div class="result-item">
                        <div class="result-title">${result.title || `Result ${index + 1}`}</div>
                        <div class="result-content">${result.content || result.description || 'No content available'}</div>
                        <div class="result-meta">
                            ${result.context_type ? `Type: ${result.context_type} | ` : ''}
                            ${result.relevance_score ? `Relevance: ${Math.round(result.relevance_score * 100)}% | ` : ''}
                            ${result.created_at ? `Created: ${new Date(result.created_at).toLocaleDateString()}` : ''}
                        </div>
                    </div>
                `).join('')}
            </body>
            </html>
        `;
    }

    private getHealthClass(score: number): string {
        if (score >= 0.8) return 'health-good';
        if (score >= 0.6) return 'health-warning';
        return 'health-error';
    }
}