import * as vscode from 'vscode';
import { ContextEngineClient, ContextSuggestion } from './contextEngineClient';

export class ContextSuggestionsProvider implements vscode.TreeDataProvider<SuggestionTreeItem> {
    private _onDidChangeTreeData: vscode.EventEmitter<SuggestionTreeItem | undefined | null | void> = new vscode.EventEmitter<SuggestionTreeItem | undefined | null | void>();
    readonly onDidChangeTreeData: vscode.Event<SuggestionTreeItem | undefined | null | void> = this._onDidChangeTreeData.event;

    private suggestions: ContextSuggestion[] = [];

    constructor(private contextClient: ContextEngineClient) {
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

    refresh(): void {
        this.loadSuggestions();
        this._onDidChangeTreeData.fire();
    }

    private async loadSuggestions(): Promise<void> {
        try {
            this.suggestions = await this.contextClient.getSuggestions();
        } catch (error) {
            console.error('[ContextSuggestionsProvider] Failed to load suggestions:', error);
            this.suggestions = [];
        }
    }

    getTreeItem(element: SuggestionTreeItem): vscode.TreeItem {
        return element;
    }

    getChildren(element?: SuggestionTreeItem): Thenable<SuggestionTreeItem[]> {
        if (!element) {
            // Root level - group suggestions by priority and file
            return Promise.resolve(this.getRootItems());
        } else if (element.contextValue === 'priorityGroup') {
            // Priority group - show suggestions for this priority
            return Promise.resolve(this.getSuggestionsForPriority(element.priority!));
        } else if (element.contextValue === 'fileGroup') {
            // File group - show suggestions for this file
            return Promise.resolve(this.getSuggestionsForFile(element.filePath!));
        } else {
            // Suggestion item - show actions
            return Promise.resolve(this.getActionsForSuggestion(element.suggestion!));
        }
    }

    private getRootItems(): SuggestionTreeItem[] {
        const items: SuggestionTreeItem[] = [];

        // Group by priority
        const priorityGroups = this.groupSuggestionsByPriority();
        for (const [priority, suggestions] of priorityGroups) {
            if (suggestions.length > 0) {
                items.push(new SuggestionTreeItem(
                    `${priority} Priority (${suggestions.length})`,
                    vscode.TreeItemCollapsibleState.Expanded,
                    'priorityGroup',
                    undefined,
                    priority
                ));
            }
        }

        // Group by file
        const fileGroups = this.groupSuggestionsByFile();
        if (fileGroups.size > 0) {
            items.push(new SuggestionTreeItem(
                'By File',
                vscode.TreeItemCollapsibleState.Collapsed,
                'fileGroupContainer'
            ));
        }

        return items;
    }

    private getSuggestionsForPriority(priority: string): SuggestionTreeItem[] {
        return this.suggestions
            .filter(s => s.priority === priority)
            .map(suggestion => new SuggestionTreeItem(
                suggestion.title,
                vscode.TreeItemCollapsibleState.Collapsed,
                'suggestion',
                suggestion
            ));
    }

    private getSuggestionsForFile(filePath: string): SuggestionTreeItem[] {
        return this.suggestions
            .filter(s => s.file_path === filePath)
            .map(suggestion => new SuggestionTreeItem(
                suggestion.title,
                vscode.TreeItemCollapsibleState.Collapsed,
                'suggestion',
                suggestion
            ));
    }

    private getActionsForSuggestion(suggestion: ContextSuggestion): SuggestionTreeItem[] {
        return suggestion.actions.map(action => new SuggestionTreeItem(
            action.title,
            vscode.TreeItemCollapsibleState.None,
            'action',
            suggestion,
            undefined,
            undefined,
            action
        ));
    }

    private groupSuggestionsByPriority(): Map<string, ContextSuggestion[]> {
        const groups = new Map<string, ContextSuggestion[]>();
        const priorities = ['Critical', 'High', 'Medium', 'Low'];

        for (const priority of priorities) {
            groups.set(priority, []);
        }

        for (const suggestion of this.suggestions) {
            const priority = suggestion.priority || 'Low';
            if (!groups.has(priority)) {
                groups.set(priority, []);
            }
            groups.get(priority)!.push(suggestion);
        }

        return groups;
    }

    private groupSuggestionsByFile(): Map<string, ContextSuggestion[]> {
        const groups = new Map<string, ContextSuggestion[]>();

        for (const suggestion of this.suggestions) {
            if (suggestion.file_path) {
                if (!groups.has(suggestion.file_path)) {
                    groups.set(suggestion.file_path, []);
                }
                groups.get(suggestion.file_path)!.push(suggestion);
            }
        }

        return groups;
    }
}

export class SuggestionTreeItem extends vscode.TreeItem {
    constructor(
        public readonly label: string,
        public readonly collapsibleState: vscode.TreeItemCollapsibleState,
        public readonly contextValue: string,
        public readonly suggestion?: ContextSuggestion,
        public readonly priority?: string,
        public readonly filePath?: string,
        public readonly action?: any
    ) {
        super(label, collapsibleState);

        this.tooltip = this.getTooltip();
        this.description = this.getDescription();
        this.iconPath = this.getIconPath();
        this.command = this.getCommand();
    }

    private getTooltip(): string {
        if (this.suggestion) {
            return this.suggestion.description;
        } else if (this.action) {
            return this.action.description;
        } else if (this.contextValue === 'priorityGroup') {
            return `Suggestions with ${this.priority} priority`;
        } else if (this.contextValue === 'fileGroup') {
            return `Suggestions for ${this.filePath}`;
        }
        return this.label;
    }

    private getDescription(): string | undefined {
        if (this.suggestion) {
            if (this.suggestion.file_path) {
                const fileName = this.suggestion.file_path.split(/[/\\]/).pop() || '';
                return fileName;
            }
        } else if (this.contextValue === 'fileGroup' && this.filePath) {
            return this.filePath.split(/[/\\]/).pop() || '';
        }
        return undefined;
    }

    private getIconPath(): vscode.ThemeIcon | undefined {
        switch (this.contextValue) {
            case 'priorityGroup':
                return this.getPriorityIcon(this.priority!);
            case 'fileGroup':
            case 'fileGroupContainer':
                return new vscode.ThemeIcon('file');
            case 'suggestion':
                return this.getSuggestionIcon(this.suggestion!);
            case 'action':
                return this.getActionIcon(this.action);
            default:
                return undefined;
        }
    }

    private getPriorityIcon(priority: string): vscode.ThemeIcon {
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

    private getSuggestionIcon(suggestion: ContextSuggestion): vscode.ThemeIcon {
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

    private getActionIcon(action: any): vscode.ThemeIcon {
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

    private getCommand(): vscode.Command | undefined {
        if (this.contextValue === 'suggestion') {
            return {
                command: 'contextEngine.showSuggestionDetails',
                title: 'Show Details',
                arguments: [this.suggestion]
            };
        } else if (this.contextValue === 'action') {
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
        } else if (this.contextValue === 'fileGroup' && this.filePath) {
            return {
                command: 'vscode.open',
                title: 'Open File',
                arguments: [vscode.Uri.file(this.filePath)]
            };
        }
        return undefined;
    }
}