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
exports.SuggestionTreeItem = exports.ContextSuggestionsProvider = void 0;
const vscode = __importStar(require("vscode"));
class ContextSuggestionsProvider {
    constructor(contextClient) {
        this.contextClient = contextClient;
        this._onDidChangeTreeData = new vscode.EventEmitter();
        this.onDidChangeTreeData = this._onDidChangeTreeData.event;
        this.suggestions = [];
        // Listen for suggestion updates
        this.contextClient.on('suggestionsUpdated', () => {
            this.refresh();
        });
        // Listen for analysis completion
        this.contextClient.on('analysisComplete', () => {
            this.refresh();
        });
        // Initial load
        this.loadSuggestions();
    }
    refresh() {
        this.loadSuggestions();
        this._onDidChangeTreeData.fire();
    }
    async loadSuggestions() {
        try {
            this.suggestions = await this.contextClient.getSuggestions();
        }
        catch (error) {
            console.error('[ContextSuggestionsProvider] Failed to load suggestions:', error);
            this.suggestions = [];
        }
    }
    getTreeItem(element) {
        return element;
    }
    getChildren(element) {
        if (!element) {
            // Root level - group suggestions by priority and file
            return Promise.resolve(this.getRootItems());
        }
        else if (element.contextValue === 'priorityGroup') {
            // Priority group - show suggestions for this priority
            return Promise.resolve(this.getSuggestionsForPriority(element.priority));
        }
        else if (element.contextValue === 'fileGroup') {
            // File group - show suggestions for this file
            return Promise.resolve(this.getSuggestionsForFile(element.filePath));
        }
        else {
            // Suggestion item - show actions
            return Promise.resolve(this.getActionsForSuggestion(element.suggestion));
        }
    }
    getRootItems() {
        const items = [];
        // Group by priority
        const priorityGroups = this.groupSuggestionsByPriority();
        for (const [priority, suggestions] of priorityGroups) {
            if (suggestions.length > 0) {
                items.push(new SuggestionTreeItem(`${priority} Priority (${suggestions.length})`, vscode.TreeItemCollapsibleState.Expanded, 'priorityGroup', undefined, priority));
            }
        }
        // Group by file
        const fileGroups = this.groupSuggestionsByFile();
        if (fileGroups.size > 0) {
            items.push(new SuggestionTreeItem('By File', vscode.TreeItemCollapsibleState.Collapsed, 'fileGroupContainer'));
        }
        return items;
    }
    getSuggestionsForPriority(priority) {
        return this.suggestions
            .filter(s => s.priority === priority)
            .map(suggestion => new SuggestionTreeItem(suggestion.title, vscode.TreeItemCollapsibleState.Collapsed, 'suggestion', suggestion));
    }
    getSuggestionsForFile(filePath) {
        return this.suggestions
            .filter(s => s.file_path === filePath)
            .map(suggestion => new SuggestionTreeItem(suggestion.title, vscode.TreeItemCollapsibleState.Collapsed, 'suggestion', suggestion));
    }
    getActionsForSuggestion(suggestion) {
        return suggestion.actions.map(action => new SuggestionTreeItem(action.title, vscode.TreeItemCollapsibleState.None, 'action', suggestion, undefined, undefined, action));
    }
    groupSuggestionsByPriority() {
        const groups = new Map();
        const priorities = ['Critical', 'High', 'Medium', 'Low'];
        for (const priority of priorities) {
            groups.set(priority, []);
        }
        for (const suggestion of this.suggestions) {
            const priority = suggestion.priority || 'Low';
            if (!groups.has(priority)) {
                groups.set(priority, []);
            }
            groups.get(priority).push(suggestion);
        }
        return groups;
    }
    groupSuggestionsByFile() {
        const groups = new Map();
        for (const suggestion of this.suggestions) {
            if (suggestion.file_path) {
                if (!groups.has(suggestion.file_path)) {
                    groups.set(suggestion.file_path, []);
                }
                groups.get(suggestion.file_path).push(suggestion);
            }
        }
        return groups;
    }
}
exports.ContextSuggestionsProvider = ContextSuggestionsProvider;
class SuggestionTreeItem extends vscode.TreeItem {
    constructor(label, collapsibleState, contextValue, suggestion, priority, filePath, action) {
        super(label, collapsibleState);
        this.label = label;
        this.collapsibleState = collapsibleState;
        this.contextValue = contextValue;
        this.suggestion = suggestion;
        this.priority = priority;
        this.filePath = filePath;
        this.action = action;
        this.tooltip = this.getTooltip();
        this.description = this.getDescription();
        this.iconPath = this.getIconPath();
        this.command = this.getCommand();
    }
    getTooltip() {
        if (this.suggestion) {
            return this.suggestion.description;
        }
        else if (this.action) {
            return this.action.description;
        }
        else if (this.contextValue === 'priorityGroup') {
            return `Suggestions with ${this.priority} priority`;
        }
        else if (this.contextValue === 'fileGroup') {
            return `Suggestions for ${this.filePath}`;
        }
        return this.label;
    }
    getDescription() {
        if (this.suggestion) {
            if (this.suggestion.file_path) {
                const fileName = this.suggestion.file_path.split(/[/\\]/).pop() || '';
                return fileName;
            }
        }
        else if (this.contextValue === 'fileGroup' && this.filePath) {
            return this.filePath.split(/[/\\]/).pop() || '';
        }
        return undefined;
    }
    getIconPath() {
        switch (this.contextValue) {
            case 'priorityGroup':
                return this.getPriorityIcon(this.priority);
            case 'fileGroup':
            case 'fileGroupContainer':
                return new vscode.ThemeIcon('file');
            case 'suggestion':
                return this.getSuggestionIcon(this.suggestion);
            case 'action':
                return this.getActionIcon(this.action);
            default:
                return undefined;
        }
    }
    getPriorityIcon(priority) {
        switch (priority) {
            case 'Critical':
                return new vscode.ThemeIcon('error', new vscode.ThemeColor('errorForeground'));
            case 'High':
                return new vscode.ThemeIcon('warning', new vscode.ThemeColor('warningForeground'));
            case 'Medium':
                return new vscode.ThemeIcon('info', new vscode.ThemeColor('infoForeground'));
            case 'Low':
            default:
                return new vscode.ThemeIcon('lightbulb', new vscode.ThemeColor('foreground'));
        }
    }
    getSuggestionIcon(suggestion) {
        switch (suggestion.suggestion_type) {
            case 'MissingDocumentation':
                return new vscode.ThemeIcon('book');
            case 'ArchitecturalDecision':
                return new vscode.ThemeIcon('organization');
            case 'BusinessRule':
                return new vscode.ThemeIcon('law');
            case 'PerformanceRequirement':
                return new vscode.ThemeIcon('dashboard');
            case 'SecurityConcern':
                return new vscode.ThemeIcon('shield');
            case 'TestCoverage':
                return new vscode.ThemeIcon('beaker');
            case 'CodePattern':
                return new vscode.ThemeIcon('code');
            case 'Refactoring':
                return new vscode.ThemeIcon('wrench');
            default:
                return new vscode.ThemeIcon('lightbulb');
        }
    }
    getActionIcon(action) {
        switch (action.action_type) {
            case 'CreateContext':
                return new vscode.ThemeIcon('add');
            case 'UpdateContext':
                return new vscode.ThemeIcon('edit');
            case 'NavigateToCode':
                return new vscode.ThemeIcon('go-to-file');
            case 'ShowDocumentation':
                return new vscode.ThemeIcon('book');
            case 'RunAnalysis':
                return new vscode.ThemeIcon('search');
            case 'ApplyFix':
                return new vscode.ThemeIcon('check');
            default:
                return new vscode.ThemeIcon('gear');
        }
    }
    getCommand() {
        if (this.contextValue === 'suggestion') {
            return {
                command: 'contextEngine.showSuggestionDetails',
                title: 'Show Details',
                arguments: [this.suggestion]
            };
        }
        else if (this.contextValue === 'action') {
            return {
                command: 'contextEngine.executeSuggestionAction',
                title: 'Execute Action',
                arguments: [{
                        suggestionId: this.suggestion?.id,
                        actionType: this.action.action_type,
                        suggestion: this.suggestion,
                        action: this.action
                    }]
            };
        }
        else if (this.contextValue === 'fileGroup' && this.filePath) {
            return {
                command: 'vscode.open',
                title: 'Open File',
                arguments: [vscode.Uri.file(this.filePath)]
            };
        }
        return undefined;
    }
}
exports.SuggestionTreeItem = SuggestionTreeItem;
//# sourceMappingURL=contextSuggestionsProvider.js.map