import * as vscode from 'vscode';
import { ContextEngineClient, ContextSuggestion } from './contextEngineClient';
import { ConfigurationManager } from './configurationManager';

export class SuggestionProvider implements vscode.HoverProvider, vscode.CodeActionProvider, vscode.CompletionItemProvider {
    private suggestionCache: Map<string, ContextSuggestion[]> = new Map();
    private cacheTimeout = 30000; // 30 seconds cache timeout

    constructor(
        private contextClient: ContextEngineClient,
        private configManager: ConfigurationManager
    ) {
        // Listen for suggestion updates from the context client
        this.contextClient.on('suggestionsUpdated', (data: any) => {
            if (data.file_path) {
                this.updateSuggestionCache(data.file_path, data.suggestions);
            }
        });

        // Clear cache periodically
        setInterval(() => {
            this.clearExpiredCache();
        }, this.cacheTimeout);
    }

    // Hover Provider Implementation
    async provideHover(
        document: vscode.TextDocument,
        position: vscode.Position,
        token: vscode.CancellationToken
    ): Promise<vscode.Hover | null> {
        if (!this.configManager.isHoverSuggestionsEnabled()) {
            return null;
        }

        const filePath = document.uri.fsPath;
        const suggestions = await this.getSuggestionsForFile(filePath);
        
        if (suggestions.length === 0) {
            return null;
        }

        // Find suggestions relevant to the current position
        const relevantSuggestions = suggestions.filter(suggestion => {
            if (suggestion.line_number) {
                // Check if the hover position is near the suggestion line
                const suggestionLine = suggestion.line_number - 1; // Convert to 0-based
                return Math.abs(position.line - suggestionLine) <= 2; // Within 2 lines
            }
            return true; // Include suggestions without specific line numbers
        });

        if (relevantSuggestions.length === 0) {
            return null;
        }

        // Create hover content
        const hoverContent = this.createHoverContent(relevantSuggestions);
        const range = document.getWordRangeAtPosition(position);
        
        return new vscode.Hover(hoverContent, range);
    }

    // Code Action Provider Implementation
    async provideCodeActions(
        document: vscode.TextDocument,
        range: vscode.Range | vscode.Selection,
        context: vscode.CodeActionContext,
        token: vscode.CancellationToken
    ): Promise<vscode.CodeAction[]> {
        if (!this.configManager.isCodeActionsEnabled()) {
            return [];
        }

        const filePath = document.uri.fsPath;
        const suggestions = await this.getSuggestionsForFile(filePath);
        
        if (suggestions.length === 0) {
            return [];
        }

        // Find suggestions relevant to the current range
        const relevantSuggestions = suggestions.filter(suggestion => {
            if (suggestion.line_number) {
                const suggestionLine = suggestion.line_number - 1; // Convert to 0-based
                return range.start.line <= suggestionLine && suggestionLine <= range.end.line;
            }
            return true; // Include suggestions without specific line numbers
        });

        // Convert suggestions to code actions
        const codeActions: vscode.CodeAction[] = [];
        
        for (const suggestion of relevantSuggestions) {
            for (const action of suggestion.actions) {
                const codeAction = this.createCodeAction(suggestion, action, document, range);
                if (codeAction) {
                    codeActions.push(codeAction);
                }
            }
        }

        return codeActions;
    }

    private async getSuggestionsForFile(filePath: string): Promise<ContextSuggestion[]> {
        // Check cache first
        const cached = this.suggestionCache.get(filePath);
        if (cached) {
            return cached;
        }

        try {
            // Fetch suggestions from context client
            const suggestions = await this.contextClient.getSuggestions(filePath);
            this.updateSuggestionCache(filePath, suggestions);
            return suggestions;
        } catch (error) {
            console.error(`[SuggestionProvider] Failed to get suggestions for ${filePath}:`, error);
            return [];
        }
    }

    private updateSuggestionCache(filePath: string, suggestions: ContextSuggestion[]): void {
        this.suggestionCache.set(filePath, suggestions);
        
        // Set timeout to clear this cache entry
        setTimeout(() => {
            this.suggestionCache.delete(filePath);
        }, this.cacheTimeout);
    }

    private clearExpiredCache(): void {
        // This is handled by individual timeouts, but we could implement
        // more sophisticated cache management here if needed
    }

    private createHoverContent(suggestions: ContextSuggestion[]): vscode.MarkdownString[] {
        const content: vscode.MarkdownString[] = [];
        
        for (const suggestion of suggestions.slice(0, 3)) { // Limit to 3 suggestions in hover
            const markdown = new vscode.MarkdownString();
            markdown.isTrusted = true;
            
            // Add suggestion title and description
            markdown.appendMarkdown(`**${suggestion.title}**\n\n`);
            markdown.appendMarkdown(`${suggestion.description}\n\n`);
            
            // Add priority badge
            const priorityColor = this.getPriorityColor(suggestion.priority);
            markdown.appendMarkdown(`Priority: <span style="color: ${priorityColor};">${suggestion.priority}</span>\n\n`);
            
            // Add available actions
            if (suggestion.actions.length > 0) {
                markdown.appendMarkdown(`**Available Actions:**\n`);
                for (const action of suggestion.actions) {
                    const command = `command:contextEngine.executeSuggestionAction?${encodeURIComponent(JSON.stringify({ suggestionId: suggestion.id, actionType: action.action_type }))}`;
                    markdown.appendMarkdown(`- [${action.title}](${command})\n`);
                }
            }
            
            content.push(markdown);
        }
        
        return content;
    }

    private createCodeAction(
        suggestion: ContextSuggestion,
        action: any,
        document: vscode.TextDocument,
        range: vscode.Range
    ): vscode.CodeAction | null {
        const codeAction = new vscode.CodeAction(
            `${suggestion.title}: ${action.title}`,
            this.getCodeActionKind(action.action_type)
        );
        
        (codeAction as any).detail = action.description;
        
        // Set the command to execute
        codeAction.command = {
            command: 'contextEngine.executeSuggestionAction',
            title: action.title,
            arguments: [{ suggestionId: suggestion.id, actionType: action.action_type, suggestion, action }]
        };
        
        // Set diagnostics if this is a fix action
        if (action.action_type === 'ApplyFix') {
            codeAction.diagnostics = this.createDiagnosticsForSuggestion(suggestion, document, range);
        }
        
        // Set as preferred if high priority
        if (suggestion.priority === 'High' || suggestion.priority === 'Critical') {
            codeAction.isPreferred = true;
        }
        
        return codeAction;
    }

    private getCodeActionKind(actionType: string): vscode.CodeActionKind {
        switch (actionType) {
            case 'ApplyFix':
                return vscode.CodeActionKind.QuickFix;
            case 'CreateContext':
            case 'UpdateContext':
                return vscode.CodeActionKind.Refactor;
            case 'RunAnalysis':
                return vscode.CodeActionKind.Source;
            default:
                return vscode.CodeActionKind.Empty;
        }
    }

    private createDiagnosticsForSuggestion(
        suggestion: ContextSuggestion,
        document: vscode.TextDocument,
        range: vscode.Range
    ): vscode.Diagnostic[] {
        const diagnostic = new vscode.Diagnostic(
            range,
            suggestion.description,
            this.getDiagnosticSeverity(suggestion.priority)
        );
        
        diagnostic.source = 'Context Engine';
        diagnostic.code = suggestion.suggestion_type;
        
        return [diagnostic];
    }

    private getDiagnosticSeverity(priority: string): vscode.DiagnosticSeverity {
        switch (priority) {
            case 'Critical':
                return vscode.DiagnosticSeverity.Error;
            case 'High':
                return vscode.DiagnosticSeverity.Warning;
            case 'Medium':
                return vscode.DiagnosticSeverity.Information;
            case 'Low':
            default:
                return vscode.DiagnosticSeverity.Hint;
        }
    }

    private getPriorityColor(priority: string): string {
        switch (priority) {
            case 'Critical':
                return '#ff0000';
            case 'High':
                return '#ff8800';
            case 'Medium':
                return '#ffaa00';
            case 'Low':
            default:
                return '#888888';
        }
    }

    // Public method to refresh suggestions for a file
    public async refreshSuggestions(filePath: string): Promise<void> {
        this.suggestionCache.delete(filePath);
        await this.getSuggestionsForFile(filePath);
    }

    // Public method to clear all cached suggestions
    public clearCache(): void {
        this.suggestionCache.clear();
    }

    // Completion provider implementation for intelligent suggestions
    async provideCompletionItems(
        document: vscode.TextDocument,
        position: vscode.Position,
        token: vscode.CancellationToken,
        context: vscode.CompletionContext
    ): Promise<vscode.CompletionItem[]> {
        if (!this.configManager.isRealTimeSuggestionsEnabled()) {
            return [];
        }

        try {
            // Get context around the cursor
            const line = document.lineAt(position);
            const textBeforeCursor = line.text.substring(0, position.character);
            const textAfterCursor = line.text.substring(position.character);

            // Get intelligent suggestions from context engine
            const suggestions = await this.getIntelligentSuggestions(
                document.uri.fsPath,
                position.line,
                position.character,
                textBeforeCursor,
                textAfterCursor
            );

            return suggestions.map(suggestion => this.createCompletionItem(suggestion));
        } catch (error) {
            console.error('[SuggestionProvider] Failed to provide completions:', error);
            return [];
        }
    }

    private async getIntelligentSuggestions(
        filePath: string,
        line: number,
        character: number,
        textBefore: string,
        textAfter: string
    ): Promise<any[]> {
        try {
            const response = await this.contextClient.queryContext(JSON.stringify({
                type: 'completion',
                filePath,
                line,
                character,
                textBefore,
                textAfter,
                language: this.getLanguageFromPath(filePath)
            }));

            return response.filter(item => item.type === 'completion_suggestion');
        } catch (error) {
            console.error('[SuggestionProvider] Failed to get intelligent suggestions:', error);
            return [];
        }
    }

    private createCompletionItem(suggestion: any): vscode.CompletionItem {
        const item = new vscode.CompletionItem(
            suggestion.label || suggestion.title,
            this.getCompletionKind(suggestion.suggestionType)
        );

        item.detail = suggestion.detail || suggestion.description;
        item.documentation = new vscode.MarkdownString(suggestion.documentation || suggestion.description);
        item.insertText = suggestion.insertText || suggestion.label;
        item.sortText = suggestion.priority === 'High' ? '0' : suggestion.priority === 'Medium' ? '1' : '2';
        
        if (suggestion.snippet) {
            item.insertText = new vscode.SnippetString(suggestion.snippet);
        }

        if (suggestion.additionalTextEdits) {
            item.additionalTextEdits = suggestion.additionalTextEdits.map((edit: any) => 
                new vscode.TextEdit(
                    new vscode.Range(edit.range.start.line, edit.range.start.character, edit.range.end.line, edit.range.end.character),
                    edit.newText
                )
            );
        }

        return item;
    }

    private getCompletionKind(suggestionType: string): vscode.CompletionItemKind {
        switch (suggestionType) {
            case 'function': return vscode.CompletionItemKind.Function;
            case 'method': return vscode.CompletionItemKind.Method;
            case 'variable': return vscode.CompletionItemKind.Variable;
            case 'class': return vscode.CompletionItemKind.Class;
            case 'interface': return vscode.CompletionItemKind.Interface;
            case 'module': return vscode.CompletionItemKind.Module;
            case 'property': return vscode.CompletionItemKind.Property;
            case 'keyword': return vscode.CompletionItemKind.Keyword;
            case 'snippet': return vscode.CompletionItemKind.Snippet;
            case 'text': return vscode.CompletionItemKind.Text;
            default: return vscode.CompletionItemKind.Text;
        }
    }

    private getLanguageFromPath(filePath: string): string {
        const extension = filePath.split('.').pop()?.toLowerCase();
        switch (extension) {
            case 'rs': return 'rust';
            case 'ts': return 'typescript';
            case 'js': return 'javascript';
            case 'py': return 'python';
            case 'java': return 'java';
            case 'cpp':
            case 'cc':
            case 'cxx': return 'cpp';
            case 'cs': return 'csharp';
            default: return 'text';
        }
    }
}