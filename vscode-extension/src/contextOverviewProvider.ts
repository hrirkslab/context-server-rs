import * as vscode from 'vscode';
import { ContextEngineClient } from './contextEngineClient';

export interface ProjectOverview {
    projectName: string;
    totalContextItems: number;
    healthScore: number;
    lastAnalysis: Date;
    activeFiles: number;
    suggestions: number;
    teamMembers?: number;
    syncStatus: 'connected' | 'disconnected' | 'syncing';
}

export class ContextOverviewProvider implements vscode.TreeDataProvider<OverviewTreeItem> {
    private _onDidChangeTreeData: vscode.EventEmitter<OverviewTreeItem | undefined | null | void> = new vscode.EventEmitter<OverviewTreeItem | undefined | null | void>();
    readonly onDidChangeTreeData: vscode.Event<OverviewTreeItem | undefined | null | void> = this._onDidChangeTreeData.event;

    private overview: ProjectOverview | null = null;

    constructor(private contextClient: ContextEngineClient) {
        // Listen for real-time updates
        this.contextClient.on('contextUpdated', () => this.refresh());
        this.contextClient.on('analysisComplete', () => this.refresh());
        this.contextClient.on('syncStatusChanged', () => this.refresh());
        
        // Initial load
        this.loadOverview();
    }

    refresh(): void {
        this.loadOverview();
        this._onDidChangeTreeData.fire();
    }

    private async loadOverview(): Promise<void> {
        try {
            const [health, insights] = await Promise.all([
                this.contextClient.getContextHealth(),
                this.contextClient.getProjectInsights()
            ]);

            this.overview = {
                projectName: vscode.workspace.workspaceFolders?.[0]?.name || 'Unknown Project',
                totalContextItems: insights.totalContextItems || 0,
                healthScore: health.overallScore || 0,
                lastAnalysis: new Date(insights.lastAnalysis || Date.now()),
                activeFiles: insights.activeFiles || 0,
                suggestions: insights.pendingSuggestions || 0,
                teamMembers: insights.teamMembers,
                syncStatus: this.contextClient.isConnected() ? 'connected' : 'disconnected'
            };
        } catch (error) {
            console.error('[ContextOverviewProvider] Failed to load overview:', error);
            this.overview = null;
        }
    }

    getTreeItem(element: OverviewTreeItem): vscode.TreeItem {
        return element;
    }

    getChildren(element?: OverviewTreeItem): Thenable<OverviewTreeItem[]> {
        if (!element) {
            return Promise.resolve(this.getRootItems());
        }
        return Promise.resolve([]);
    }

    private getRootItems(): OverviewTreeItem[] {
        if (!this.overview) {
            return [new OverviewTreeItem(
                'Loading project overview...',
                vscode.TreeItemCollapsibleState.None,
                'loading',
                new vscode.ThemeIcon('loading~spin')
            )];
        }

        const items: OverviewTreeItem[] = [];

        // Project header
        items.push(new OverviewTreeItem(
            this.overview.projectName,
            vscode.TreeItemCollapsibleState.None,
            'projectName',
            new vscode.ThemeIcon('folder-opened'),
            undefined,
            `Active project: ${this.overview.projectName}`
        ));

        // Health score with color coding
        const healthIcon = this.getHealthIcon(this.overview.healthScore);
        const healthColor = this.getHealthColor(this.overview.healthScore);
        items.push(new OverviewTreeItem(
            `Health Score: ${Math.round(this.overview.healthScore * 100)}%`,
            vscode.TreeItemCollapsibleState.None,
            'healthScore',
            new vscode.ThemeIcon(healthIcon, healthColor),
            `${Math.round(this.overview.healthScore * 100)}%`,
            `Overall context health: ${this.getHealthDescription(this.overview.healthScore)}`
        ));

        // Context items count
        items.push(new OverviewTreeItem(
            'Context Items',
            vscode.TreeItemCollapsibleState.None,
            'contextCount',
            new vscode.ThemeIcon('database'),
            this.overview.totalContextItems.toString(),
            `Total context items in project: ${this.overview.totalContextItems}`
        ));

        // Active files
        items.push(new OverviewTreeItem(
            'Active Files',
            vscode.TreeItemCollapsibleState.None,
            'activeFiles',
            new vscode.ThemeIcon('file-code'),
            this.overview.activeFiles.toString(),
            `Files with context analysis: ${this.overview.activeFiles}`
        ));

        // Pending suggestions
        const suggestionIcon = this.overview.suggestions > 0 ? 'lightbulb' : 'check';
        const suggestionColor = this.overview.suggestions > 0 ? new vscode.ThemeColor('warningForeground') : new vscode.ThemeColor('foreground');
        items.push(new OverviewTreeItem(
            'Suggestions',
            vscode.TreeItemCollapsibleState.None,
            'suggestions',
            new vscode.ThemeIcon(suggestionIcon, suggestionColor),
            this.overview.suggestions.toString(),
            `Pending suggestions: ${this.overview.suggestions}`,
            this.overview.suggestions > 0 ? {
                command: 'contextEngine.showSuggestions',
                title: 'Show Suggestions'
            } : undefined
        ));

        // Sync status
        const syncIcon = this.getSyncIcon(this.overview.syncStatus);
        const syncColor = this.getSyncColor(this.overview.syncStatus);
        items.push(new OverviewTreeItem(
            'Sync Status',
            vscode.TreeItemCollapsibleState.None,
            'syncStatus',
            new vscode.ThemeIcon(syncIcon, syncColor),
            this.overview.syncStatus,
            `Real-time sync: ${this.overview.syncStatus}`
        ));

        // Team members (if team mode enabled)
        if (this.overview.teamMembers !== undefined) {
            items.push(new OverviewTreeItem(
                'Team Members',
                vscode.TreeItemCollapsibleState.None,
                'teamMembers',
                new vscode.ThemeIcon('organization'),
                this.overview.teamMembers.toString(),
                `Active team members: ${this.overview.teamMembers}`,
                {
                    command: 'contextEngine.showTeamActivity',
                    title: 'Show Team Activity'
                }
            ));
        }

        // Last analysis
        const timeAgo = this.getTimeAgo(this.overview.lastAnalysis);
        items.push(new OverviewTreeItem(
            'Last Analysis',
            vscode.TreeItemCollapsibleState.None,
            'lastAnalysis',
            new vscode.ThemeIcon('clock'),
            timeAgo,
            `Last context analysis: ${this.overview.lastAnalysis.toLocaleString()}`
        ));

        return items;
    }

    private getHealthIcon(score: number): string {
        if (score >= 0.8) return 'check-all';
        if (score >= 0.6) return 'check';
        if (score >= 0.4) return 'warning';
        return 'error';
    }

    private getHealthColor(score: number): vscode.ThemeColor {
        if (score >= 0.8) return new vscode.ThemeColor('testing.iconPassed');
        if (score >= 0.6) return new vscode.ThemeColor('infoForeground');
        if (score >= 0.4) return new vscode.ThemeColor('warningForeground');
        return new vscode.ThemeColor('errorForeground');
    }

    private getHealthDescription(score: number): string {
        if (score >= 0.8) return 'Excellent';
        if (score >= 0.6) return 'Good';
        if (score >= 0.4) return 'Needs Attention';
        return 'Poor';
    }

    private getSyncIcon(status: string): string {
        switch (status) {
            case 'connected': return 'check';
            case 'syncing': return 'sync~spin';
            case 'disconnected': return 'error';
            default: return 'question';
        }
    }

    private getSyncColor(status: string): vscode.ThemeColor {
        switch (status) {
            case 'connected': return new vscode.ThemeColor('testing.iconPassed');
            case 'syncing': return new vscode.ThemeColor('infoForeground');
            case 'disconnected': return new vscode.ThemeColor('errorForeground');
            default: return new vscode.ThemeColor('foreground');
        }
    }

    private getTimeAgo(date: Date): string {
        const now = new Date();
        const diffMs = now.getTime() - date.getTime();
        const diffMins = Math.floor(diffMs / 60000);
        const diffHours = Math.floor(diffMins / 60);
        const diffDays = Math.floor(diffHours / 24);

        if (diffMins < 1) return 'Just now';
        if (diffMins < 60) return `${diffMins}m ago`;
        if (diffHours < 24) return `${diffHours}h ago`;
        return `${diffDays}d ago`;
    }
}

export class OverviewTreeItem extends vscode.TreeItem {
    constructor(
        public readonly label: string,
        public readonly collapsibleState: vscode.TreeItemCollapsibleState,
        public readonly contextValue: string,
        public readonly iconPath?: vscode.ThemeIcon,
        public readonly description?: string,
        public readonly tooltip?: string,
        public readonly command?: vscode.Command
    ) {
        super(label, collapsibleState);
        this.iconPath = iconPath;
        this.description = description;
        this.tooltip = tooltip;
        this.command = command;
    }
}