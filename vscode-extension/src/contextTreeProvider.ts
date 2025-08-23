import * as vscode from 'vscode';
import { ContextEngineClient } from './contextEngineClient';

export interface ContextItem {
    id: string;
    title: string;
    type: string;
    description: string;
    filePath?: string;
    lineNumber?: number;
    relationships: ContextRelationship[];
    qualityScore: number;
    lastModified: Date;
    tags: string[];
}

export interface ContextRelationship {
    targetId: string;
    type: 'depends_on' | 'conflicts' | 'implements' | 'extends' | 'references' | 'similar';
    strength: number;
}

export class ContextTreeProvider implements vscode.TreeDataProvider<ContextTreeItem> {
    private _onDidChangeTreeData: vscode.EventEmitter<ContextTreeItem | undefined | null | void> = new vscode.EventEmitter<ContextTreeItem | undefined | null | void>();
    readonly onDidChangeTreeData: vscode.Event<ContextTreeItem | undefined | null | void> = this._onDidChangeTreeData.event;

    private contextItems: ContextItem[] = [];
    private filteredItems: ContextItem[] = [];
    private currentFilter: string = '';
    private currentSearchTerm: string = '';
    private groupBy: 'type' | 'file' | 'quality' | 'none' = 'type';

    constructor(private contextClient: ContextEngineClient) {
        // Listen for real-time updates
        this.contextClient.on('contextUpdated', () => this.refresh());
        this.contextClient.on('analysisComplete', () => this.refresh());
        
        // Initial load
        this.loadContextItems();
    }

    refresh(): void {
        this.loadContextItems();
        this._onDidChangeTreeData.fire();
    }

    setFilter(filter: string): void {
        this.currentFilter = filter;
        this.applyFiltersAndSearch();
        this._onDidChangeTreeData.fire();
    }

    setSearchTerm(searchTerm: string): void {
        this.currentSearchTerm = searchTerm;
        this.applyFiltersAndSearch();
        this._onDidChangeTreeData.fire();
    }

    setGroupBy(groupBy: 'type' | 'file' | 'quality' | 'none'): void {
        this.groupBy = groupBy;
        this._onDidChangeTreeData.fire();
    }

    private async loadContextItems(): Promise<void> {
        try {
            const response = await this.contextClient.queryContext('*'); // Get all context items
            this.contextItems = response.map(item => ({
                id: item.id,
                title: item.title,
                type: item.context_type || 'general',
                description: item.description || '',
                filePath: item.metadata?.file_path,
                lineNumber: item.metadata?.line_number,
                relationships: item.relationships || [],
                qualityScore: item.quality_score || 0.5,
                lastModified: new Date(item.updated_at || Date.now()),
                tags: item.tags || []
            }));
            
            this.applyFiltersAndSearch();
        } catch (error) {
            console.error('[ContextTreeProvider] Failed to load context items:', error);
            this.contextItems = [];
            this.filteredItems = [];
        }
    }

    private applyFiltersAndSearch(): void {
        let items = [...this.contextItems];

        // Apply type filter
        if (this.currentFilter && this.currentFilter !== 'all') {
            items = items.filter(item => item.type === this.currentFilter);
        }

        // Apply search term
        if (this.currentSearchTerm) {
            const searchLower = this.currentSearchTerm.toLowerCase();
            items = items.filter(item => 
                item.title.toLowerCase().includes(searchLower) ||
                item.description.toLowerCase().includes(searchLower) ||
                item.tags.some(tag => tag.toLowerCase().includes(searchLower))
            );
        }

        this.filteredItems = items;
    }

    getTreeItem(element: ContextTreeItem): vscode.TreeItem {
        return element;
    }

    getChildren(element?: ContextTreeItem): Thenable<ContextTreeItem[]> {
        if (!element) {
            return Promise.resolve(this.getRootItems());
        } else if (element.contextValue === 'group') {
            return Promise.resolve(this.getGroupItems(element.groupKey!));
        } else if (element.contextValue === 'contextItem') {
            return Promise.resolve(this.getRelationshipItems(element.contextItem!));
        }
        return Promise.resolve([]);
    }

    private getRootItems(): ContextTreeItem[] {
        if (this.filteredItems.length === 0) {
            return [new ContextTreeItem(
                this.contextItems.length === 0 ? 'No context items found' : 'No items match current filter',
                vscode.TreeItemCollapsibleState.None,
                'empty',
                new vscode.ThemeIcon('info')
            )];
        }

        if (this.groupBy === 'none') {
            return this.filteredItems.map(item => this.createContextTreeItem(item));
        }

        // Group items
        const groups = this.groupItems(this.filteredItems);
        return Array.from(groups.entries()).map(([groupKey, items]) => 
            new ContextTreeItem(
                `${this.getGroupDisplayName(groupKey)} (${items.length})`,
                vscode.TreeItemCollapsibleState.Expanded,
                'group',
                this.getGroupIcon(groupKey),
                undefined,
                groupKey
            )
        );
    }

    private getGroupItems(groupKey: string): ContextTreeItem[] {
        const groups = this.groupItems(this.filteredItems);
        const items = groups.get(groupKey) || [];
        return items.map(item => this.createContextTreeItem(item));
    }

    private getRelationshipItems(contextItem: ContextItem): ContextTreeItem[] {
        if (contextItem.relationships.length === 0) {
            return [];
        }

        return contextItem.relationships.map(rel => {
            const relatedItem = this.contextItems.find(item => item.id === rel.targetId);
            const title = relatedItem ? relatedItem.title : `Unknown (${rel.targetId})`;
            
            return new ContextTreeItem(
                `${this.getRelationshipTypeIcon(rel.type)} ${title}`,
                vscode.TreeItemCollapsibleState.None,
                'relationship',
                new vscode.ThemeIcon(this.getRelationshipTypeIcon(rel.type)),
                `${Math.round(rel.strength * 100)}%`,
                undefined,
                undefined,
                relatedItem ? {
                    command: 'contextEngine.showContextDetails',
                    title: 'Show Details',
                    arguments: [relatedItem]
                } : undefined
            );
        });
    }

    private createContextTreeItem(item: ContextItem): ContextTreeItem {
        const hasRelationships = item.relationships.length > 0;
        const collapsibleState = hasRelationships ? 
            vscode.TreeItemCollapsibleState.Collapsed : 
            vscode.TreeItemCollapsibleState.None;

        return new ContextTreeItem(
            item.title,
            collapsibleState,
            'contextItem',
            this.getContextTypeIcon(item.type),
            this.getContextDescription(item),
            undefined,
            item,
            {
                command: 'contextEngine.showContextDetails',
                title: 'Show Details',
                arguments: [item]
            }
        );
    }

    private groupItems(items: ContextItem[]): Map<string, ContextItem[]> {
        const groups = new Map<string, ContextItem[]>();

        for (const item of items) {
            let groupKey: string;
            
            switch (this.groupBy) {
                case 'type':
                    groupKey = item.type;
                    break;
                case 'file':
                    groupKey = item.filePath ? 
                        item.filePath.split(/[/\\]/).pop() || 'Unknown File' : 
                        'No File';
                    break;
                case 'quality':
                    if (item.qualityScore >= 0.8) groupKey = 'High Quality';
                    else if (item.qualityScore >= 0.6) groupKey = 'Good Quality';
                    else if (item.qualityScore >= 0.4) groupKey = 'Fair Quality';
                    else groupKey = 'Needs Improvement';
                    break;
                default:
                    groupKey = 'All Items';
            }

            if (!groups.has(groupKey)) {
                groups.set(groupKey, []);
            }
            groups.get(groupKey)!.push(item);
        }

        return groups;
    }

    private getGroupDisplayName(groupKey: string): string {
        // Convert snake_case to Title Case
        return groupKey.split('_').map(word => 
            word.charAt(0).toUpperCase() + word.slice(1)
        ).join(' ');
    }

    private getGroupIcon(groupKey: string): vscode.ThemeIcon {
        switch (this.groupBy) {
            case 'type':
                return this.getContextTypeIcon(groupKey);
            case 'file':
                return new vscode.ThemeIcon('file');
            case 'quality':
                if (groupKey.includes('High')) return new vscode.ThemeIcon('star-full');
                if (groupKey.includes('Good')) return new vscode.ThemeIcon('star-half');
                if (groupKey.includes('Fair')) return new vscode.ThemeIcon('star-empty');
                return new vscode.ThemeIcon('warning');
            default:
                return new vscode.ThemeIcon('folder');
        }
    }

    private getContextTypeIcon(type: string): vscode.ThemeIcon {
        switch (type) {
            case 'business_rule': return new vscode.ThemeIcon('law');
            case 'architectural_decision': return new vscode.ThemeIcon('organization');
            case 'performance_requirement': return new vscode.ThemeIcon('dashboard');
            case 'security_policy': return new vscode.ThemeIcon('shield');
            case 'api_specification': return new vscode.ThemeIcon('globe');
            case 'data_model': return new vscode.ThemeIcon('database');
            case 'workflow': return new vscode.ThemeIcon('git-branch');
            case 'integration_point': return new vscode.ThemeIcon('plug');
            default: return new vscode.ThemeIcon('note');
        }
    }

    private getRelationshipTypeIcon(type: string): string {
        switch (type) {
            case 'depends_on': return 'arrow-right';
            case 'conflicts': return 'error';
            case 'implements': return 'check';
            case 'extends': return 'arrow-up';
            case 'references': return 'link';
            case 'similar': return 'symbol-misc';
            default: return 'question';
        }
    }

    private getContextDescription(item: ContextItem): string {
        const parts: string[] = [];
        
        if (item.qualityScore) {
            parts.push(`${Math.round(item.qualityScore * 100)}%`);
        }
        
        if (item.relationships.length > 0) {
            parts.push(`${item.relationships.length} rel`);
        }
        
        if (item.filePath) {
            const fileName = item.filePath.split(/[/\\]/).pop();
            parts.push(fileName || '');
        }

        return parts.join(' • ');
    }

    // Public methods for commands
    async showFilterDialog(): Promise<void> {
        const types = ['all', ...new Set(this.contextItems.map(item => item.type))];
        const selected = await vscode.window.showQuickPick(
            types.map(type => ({
                label: this.getGroupDisplayName(type),
                value: type
            })),
            { placeHolder: 'Select context type to filter by' }
        );

        if (selected) {
            this.setFilter(selected.value);
        }
    }

    async showSearchDialog(): Promise<void> {
        const searchTerm = await vscode.window.showInputBox({
            prompt: 'Enter search term',
            value: this.currentSearchTerm,
            placeHolder: 'Search context items...'
        });

        if (searchTerm !== undefined) {
            this.setSearchTerm(searchTerm);
        }
    }

    async showGroupByDialog(): Promise<void> {
        const options = [
            { label: 'By Type', value: 'type' as const },
            { label: 'By File', value: 'file' as const },
            { label: 'By Quality', value: 'quality' as const },
            { label: 'No Grouping', value: 'none' as const }
        ];

        const selected = await vscode.window.showQuickPick(options, {
            placeHolder: 'Select grouping method'
        });

        if (selected) {
            this.setGroupBy(selected.value);
        }
    }
}

export class ContextTreeItem extends vscode.TreeItem {
    constructor(
        public readonly label: string,
        public readonly collapsibleState: vscode.TreeItemCollapsibleState,
        public readonly contextValue: string,
        public readonly iconPath?: vscode.ThemeIcon,
        public readonly description?: string,
        public readonly groupKey?: string,
        public readonly contextItem?: ContextItem,
        public readonly command?: vscode.Command
    ) {
        super(label, collapsibleState);
        this.iconPath = iconPath;
        this.description = description;
        this.command = command;
        
        if (contextItem) {
            this.tooltip = this.createTooltip(contextItem);
        }
    }

    private createTooltip(item: ContextItem): string {
        const lines = [
            `**${item.title}**`,
            `Type: ${this.formatType(item.type)}`,
            `Quality: ${Math.round(item.qualityScore * 100)}%`,
            `Modified: ${item.lastModified.toLocaleDateString()}`,
            ''
        ];

        if (item.description) {
            lines.push(item.description);
            lines.push('');
        }

        if (item.filePath) {
            lines.push(`File: ${item.filePath}`);
            if (item.lineNumber) {
                lines.push(`Line: ${item.lineNumber}`);
            }
            lines.push('');
        }

        if (item.tags.length > 0) {
            lines.push(`Tags: ${item.tags.join(', ')}`);
            lines.push('');
        }

        if (item.relationships.length > 0) {
            lines.push(`Relationships: ${item.relationships.length}`);
        }

        return lines.join('\n');
    }

    private formatType(type: string): string {
        return type.split('_').map(word => 
            word.charAt(0).toUpperCase() + word.slice(1)
        ).join(' ');
    }
}