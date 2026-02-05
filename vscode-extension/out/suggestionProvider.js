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
exports.SuggestionProvider = void 0;
const vscode = __importStar(require("vscode"));
class SuggestionProvider {
    constructor(contextClient, configManager) {
        this.contextClient = contextClient;
        this.configManager = configManager;
        this.suggestionCache = new Map();
        this.cacheTimeout = 30000; // 30 seconds cache timeout
        // Listen for suggestion updates from the context client
        this.contextClient.on('suggestionsUpdated', (data) => {
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
    async provideHover(document, position, token) {
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
    async provideCodeActions(document, range, context, token) {
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
        const codeActions = [];
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
    async getSuggestionsForFile(filePath) {
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
        }
        catch (error) {
            console.error(`[SuggestionProvider] Failed to get suggestions for ${filePath}:`, error);
            return [];
        }
    }
    updateSuggestionCache(filePath, suggestions) {
        this.suggestionCache.set(filePath, suggestions);
        // Set timeout to clear this cache entry
        setTimeout(() => {
            this.suggestionCache.delete(filePath);
        }, this.cacheTimeout);
    }
    clearExpiredCache() {
        // This is handled by individual timeouts, but we could implement
        // more sophisticated cache management here if needed
    }
    createHoverContent(suggestions) {
        const content = [];
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
    createCodeAction(suggestion, action, document, range) {
        const codeAction = new vscode.CodeAction(`${suggestion.title}: ${action.title}`, this.getCodeActionKind(action.action_type));
        codeAction.detail = action.description;
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
    getCodeActionKind(actionType) {
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
    createDiagnosticsForSuggestion(suggestion, document, range) {
        const diagnostic = new vscode.Diagnostic(range, suggestion.description, this.getDiagnosticSeverity(suggestion.priority));
        diagnostic.source = 'Context Engine';
        diagnostic.code = suggestion.suggestion_type;
        return [diagnostic];
    }
    getDiagnosticSeverity(priority) {
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
    getPriorityColor(priority) {
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
    async refreshSuggestions(filePath) {
        this.suggestionCache.delete(filePath);
        await this.getSuggestionsForFile(filePath);
    }
    // Public method to clear all cached suggestions
    clearCache() {
        this.suggestionCache.clear();
    }
    // Completion provider implementation for intelligent suggestions
    async provideCompletionItems(document, position, token, context) {
        if (!this.configManager.isRealTimeSuggestionsEnabled()) {
            return [];
        }
        try {
            // Get context around the cursor
            const line = document.lineAt(position);
            const textBeforeCursor = line.text.substring(0, position.character);
            const textAfterCursor = line.text.substring(position.character);
            // Get intelligent suggestions from context engine
            const suggestions = await this.getIntelligentSuggestions(document.uri.fsPath, position.line, position.character, textBeforeCursor, textAfterCursor);
            return suggestions.map(suggestion => this.createCompletionItem(suggestion));
        }
        catch (error) {
            console.error('[SuggestionProvider] Failed to provide completions:', error);
            return [];
        }
    }
    async getIntelligentSuggestions(filePath, line, character, textBefore, textAfter) {
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
        }
        catch (error) {
            console.error('[SuggestionProvider] Failed to get intelligent suggestions:', error);
            return [];
        }
    }
    createCompletionItem(suggestion) {
        const item = new vscode.CompletionItem(suggestion.label || suggestion.title, this.getCompletionKind(suggestion.suggestionType));
        item.detail = suggestion.detail || suggestion.description;
        item.documentation = new vscode.MarkdownString(suggestion.documentation || suggestion.description);
        item.insertText = suggestion.insertText || suggestion.label;
        item.sortText = suggestion.priority === 'High' ? '0' : suggestion.priority === 'Medium' ? '1' : '2';
        if (suggestion.snippet) {
            item.insertText = new vscode.SnippetString(suggestion.snippet);
        }
        if (suggestion.additionalTextEdits) {
            item.additionalTextEdits = suggestion.additionalTextEdits.map((edit) => new vscode.TextEdit(new vscode.Range(edit.range.start.line, edit.range.start.character, edit.range.end.line, edit.range.end.character), edit.newText));
        }
        return item;
    }
    getCompletionKind(suggestionType) {
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
    getLanguageFromPath(filePath) {
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
exports.SuggestionProvider = SuggestionProvider;
//# sourceMappingURL=suggestionProvider.js.map