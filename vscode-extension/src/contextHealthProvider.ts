import * as vscode from 'vscode';
import { ContextEngineClient } from './contextEngineClient';

export interface HealthMetrics {
    overallScore: number;
    completeness: number;
    consistency: number;
    freshness: number;
    usage: number;
    coverage: number;
    issues: HealthIssue[];
    recommendations: HealthRecommendation[];
    trends: HealthTrend[];
}

export interface HealthIssue {
    id: string;
    severity: 'critical' | 'high' | 'medium' | 'low';
    title: string;
    description: string;
    affectedItems: number;
    suggestedAction: string;
}

export interface HealthRecommendation {
    id: string;
    priority: 'high' | 'medium' | 'low';
    title: string;
    description: string;
    impact: string;
    effort: 'low' | 'medium' | 'high';
}

export interface HealthTrend {
    metric: string;
    current: number;
    previous: number;
    change: number;
    direction: 'up' | 'down' | 'stable';
}

export class ContextHealthProvider implements vscode.TreeDataProvider<HealthTreeItem> {
    private _onDidChangeTreeData: vscode.EventEmitter<HealthTreeItem | undefined | null | void> = new vscode.EventEmitter<HealthTreeItem | undefined | null | void>();
    readonly onDidChangeTreeData: vscode.Event<HealthTreeItem | undefined | null | void> = this._onDidChangeTreeData.event;

    private healthMetrics: HealthMetrics | null = null;
    private refreshInterval: NodeJS.Timeout | null = null;

    constructor(private contextClient: ContextEngineClient) {
        // Listen for real-time updates
        this.contextClient.on('contextUpdated', () => this.refresh());
        this.contextClient.on('analysisComplete', () => this.refresh());
        
        // Initial load and periodic refresh
        this.loadHealthMetrics();
        this.startPeriodicRefresh();
    }

    refresh(): void {
        this.loadHealthMetrics();
        this._onDidChangeTreeData.fire();
    }

    dispose(): void {
        if (this.refreshInterval) {
            clearInterval(this.refreshInterval);
            this.refreshInterval = null;
        }
    }

    private startPeriodicRefresh(): void {
        // Refresh health metrics every 30 seconds
        this.refreshInterval = setInterval(() => {
            this.loadHealthMetrics();
        }, 30000);
    }

    private async loadHealthMetrics(): Promise<void> {
        try {
            const health = await this.contextClient.getContextHealth();
            this.healthMetrics = {
                overallScore: health.overallScore || 0,
                completeness: health.completeness || 0,
                consistency: health.consistency || 0,
                freshness: health.freshness || 0,
                usage: health.usage || 0,
                coverage: health.coverage || 0,
                issues: health.issues || [],
                recommendations: health.recommendations || [],
                trends: health.trends || []
            };
            this._onDidChangeTreeData.fire();
        } catch (error) {
            console.error('[ContextHealthProvider] Failed to load health metrics:', error);
            this.healthMetrics = null;
        }
    }

    getTreeItem(element: HealthTreeItem): vscode.TreeItem {
        return element;
    }

    getChildren(element?: HealthTreeItem): Thenable<HealthTreeItem[]> {
        if (!element) {
            return Promise.resolve(this.getRootItems());
        } else if (element.contextValue === 'issuesSection') {
            return Promise.resolve(this.getIssueItems());
        } else if (element.contextValue === 'recommendationsSection') {
            return Promise.resolve(this.getRecommendationItems());
        } else if (element.contextValue === 'trendsSection') {
            return Promise.resolve(this.getTrendItems());
        }
        return Promise.resolve([]);
    }

    private getRootItems(): HealthTreeItem[] {
        if (!this.healthMetrics) {
            return [new HealthTreeItem(
                'Loading health metrics...',
                vscode.TreeItemCollapsibleState.None,
                'loading',
                new vscode.ThemeIcon('loading~spin')
            )];
        }

        const items: HealthTreeItem[] = [];

        // Overall health score
        const overallIcon = this.getScoreIcon(this.healthMetrics.overallScore);
        const overallColor = this.getScoreColor(this.healthMetrics.overallScore);
        items.push(new HealthTreeItem(
            'Overall Health',
            vscode.TreeItemCollapsibleState.None,
            'overallHealth',
            new vscode.ThemeIcon(overallIcon, overallColor),
            `${Math.round(this.healthMetrics.overallScore * 100)}%`,
            `Overall context health score: ${this.getScoreDescription(this.healthMetrics.overallScore)}`
        ));

        // Individual metrics
        const metrics = [
            { key: 'completeness', label: 'Completeness', value: this.healthMetrics.completeness },
            { key: 'consistency', label: 'Consistency', value: this.healthMetrics.consistency },
            { key: 'freshness', label: 'Freshness', value: this.healthMetrics.freshness },
            { key: 'usage', label: 'Usage', value: this.healthMetrics.usage },
            { key: 'coverage', label: 'Coverage', value: this.healthMetrics.coverage }
        ];

        for (const metric of metrics) {
            const icon = this.getScoreIcon(metric.value);
            const color = this.getScoreColor(metric.value);
            const trend = this.healthMetrics.trends.find(t => t.metric === metric.key);
            const trendIcon = trend ? this.getTrendIcon(trend.direction) : '';
            
            items.push(new HealthTreeItem(
                metric.label,
                vscode.TreeItemCollapsibleState.None,
                'metric',
                new vscode.ThemeIcon(icon, color),
                `${Math.round(metric.value * 100)}% ${trendIcon}`,
                this.getMetricTooltip(metric.key, metric.value, trend)
            ));
        }

        // Issues section
        if (this.healthMetrics.issues.length > 0) {
            const criticalIssues = this.healthMetrics.issues.filter(i => i.severity === 'critical').length;
            const highIssues = this.healthMetrics.issues.filter(i => i.severity === 'high').length;
            const issueIcon = criticalIssues > 0 ? 'error' : highIssues > 0 ? 'warning' : 'info';
            const issueColor = criticalIssues > 0 ? 
                new vscode.ThemeColor('errorForeground') : 
                highIssues > 0 ? 
                    new vscode.ThemeColor('warningForeground') : 
                    new vscode.ThemeColor('infoForeground');

            items.push(new HealthTreeItem(
                'Issues',
                vscode.TreeItemCollapsibleState.Collapsed,
                'issuesSection',
                new vscode.ThemeIcon(issueIcon, issueColor),
                `${this.healthMetrics.issues.length} found`,
                `${criticalIssues} critical, ${highIssues} high priority issues`
            ));
        }

        // Recommendations section
        if (this.healthMetrics.recommendations.length > 0) {
            items.push(new HealthTreeItem(
                'Recommendations',
                vscode.TreeItemCollapsibleState.Collapsed,
                'recommendationsSection',
                new vscode.ThemeIcon('lightbulb'),
                `${this.healthMetrics.recommendations.length} available`,
                'Suggestions to improve context health'
            ));
        }

        // Trends section
        if (this.healthMetrics.trends.length > 0) {
            items.push(new HealthTreeItem(
                'Trends',
                vscode.TreeItemCollapsibleState.Collapsed,
                'trendsSection',
                new vscode.ThemeIcon('graph'),
                `${this.healthMetrics.trends.length} metrics`,
                'Historical trends and changes'
            ));
        }

        return items;
    }

    private getIssueItems(): HealthTreeItem[] {
        if (!this.healthMetrics) return [];

        return this.healthMetrics.issues.map(issue => {
            const icon = this.getSeverityIcon(issue.severity);
            const color = this.getSeverityColor(issue.severity);
            
            return new HealthTreeItem(
                issue.title,
                vscode.TreeItemCollapsibleState.None,
                'issue',
                new vscode.ThemeIcon(icon, color),
                `${issue.affectedItems} items`,
                `${issue.description}\n\nSuggested action: ${issue.suggestedAction}`,
                {
                    command: 'contextEngine.showIssueDetails',
                    title: 'Show Details',
                    arguments: [issue]
                }
            );
        });
    }

    private getRecommendationItems(): HealthTreeItem[] {
        if (!this.healthMetrics) return [];

        return this.healthMetrics.recommendations.map(rec => {
            const icon = this.getPriorityIcon(rec.priority);
            const color = this.getPriorityColor(rec.priority);
            
            return new HealthTreeItem(
                rec.title,
                vscode.TreeItemCollapsibleState.None,
                'recommendation',
                new vscode.ThemeIcon(icon, color),
                `${rec.effort} effort`,
                `${rec.description}\n\nImpact: ${rec.impact}\nEffort: ${rec.effort}`,
                {
                    command: 'contextEngine.showRecommendationDetails',
                    title: 'Show Details',
                    arguments: [rec]
                }
            );
        });
    }

    private getTrendItems(): HealthTreeItem[] {
        if (!this.healthMetrics) return [];

        return this.healthMetrics.trends.map(trend => {
            const icon = this.getTrendIcon(trend.direction);
            const color = this.getTrendColor(trend.direction);
            const changePercent = Math.round(Math.abs(trend.change) * 100);
            
            return new HealthTreeItem(
                this.formatMetricName(trend.metric),
                vscode.TreeItemCollapsibleState.None,
                'trend',
                new vscode.ThemeIcon(icon, color),
                `${trend.direction === 'up' ? '+' : trend.direction === 'down' ? '-' : ''}${changePercent}%`,
                `Current: ${Math.round(trend.current * 100)}%\nPrevious: ${Math.round(trend.previous * 100)}%\nChange: ${trend.direction === 'up' ? '+' : trend.direction === 'down' ? '-' : ''}${changePercent}%`
            );
        });
    }

    private getScoreIcon(score: number): string {
        if (score >= 0.8) return 'check-all';
        if (score >= 0.6) return 'check';
        if (score >= 0.4) return 'warning';
        return 'error';
    }

    private getScoreColor(score: number): vscode.ThemeColor {
        if (score >= 0.8) return new vscode.ThemeColor('testing.iconPassed');
        if (score >= 0.6) return new vscode.ThemeColor('infoForeground');
        if (score >= 0.4) return new vscode.ThemeColor('warningForeground');
        return new vscode.ThemeColor('errorForeground');
    }

    private getScoreDescription(score: number): string {
        if (score >= 0.8) return 'Excellent';
        if (score >= 0.6) return 'Good';
        if (score >= 0.4) return 'Needs Attention';
        return 'Poor';
    }

    private getSeverityIcon(severity: string): string {
        switch (severity) {
            case 'critical': return 'error';
            case 'high': return 'warning';
            case 'medium': return 'info';
            case 'low': return 'lightbulb';
            default: return 'question';
        }
    }

    private getSeverityColor(severity: string): vscode.ThemeColor {
        switch (severity) {
            case 'critical': return new vscode.ThemeColor('errorForeground');
            case 'high': return new vscode.ThemeColor('warningForeground');
            case 'medium': return new vscode.ThemeColor('infoForeground');
            case 'low': return new vscode.ThemeColor('foreground');
            default: return new vscode.ThemeColor('foreground');
        }
    }

    private getPriorityIcon(priority: string): string {
        switch (priority) {
            case 'high': return 'arrow-up';
            case 'medium': return 'arrow-right';
            case 'low': return 'arrow-down';
            default: return 'lightbulb';
        }
    }

    private getPriorityColor(priority: string): vscode.ThemeColor {
        switch (priority) {
            case 'high': return new vscode.ThemeColor('errorForeground');
            case 'medium': return new vscode.ThemeColor('warningForeground');
            case 'low': return new vscode.ThemeColor('infoForeground');
            default: return new vscode.ThemeColor('foreground');
        }
    }

    private getTrendIcon(direction: string): string {
        switch (direction) {
            case 'up': return 'trending-up';
            case 'down': return 'trending-down';
            case 'stable': return 'arrow-right';
            default: return 'question';
        }
    }

    private getTrendColor(direction: string): vscode.ThemeColor {
        switch (direction) {
            case 'up': return new vscode.ThemeColor('testing.iconPassed');
            case 'down': return new vscode.ThemeColor('errorForeground');
            case 'stable': return new vscode.ThemeColor('infoForeground');
            default: return new vscode.ThemeColor('foreground');
        }
    }

    private getMetricTooltip(key: string, value: number, trend?: HealthTrend): string {
        const lines = [
            `${this.formatMetricName(key)}: ${Math.round(value * 100)}%`,
            this.getMetricDescription(key)
        ];

        if (trend) {
            const changePercent = Math.round(Math.abs(trend.change) * 100);
            const direction = trend.direction === 'up' ? 'increased' : trend.direction === 'down' ? 'decreased' : 'remained stable';
            lines.push(`\nTrend: ${direction} by ${changePercent}%`);
        }

        return lines.join('\n');
    }

    private getMetricDescription(key: string): string {
        switch (key) {
            case 'completeness': return 'How complete your context documentation is';
            case 'consistency': return 'How consistent your context is across the project';
            case 'freshness': return 'How up-to-date your context information is';
            case 'usage': return 'How actively your context is being used';
            case 'coverage': return 'How much of your codebase has context coverage';
            default: return 'Context health metric';
        }
    }

    private formatMetricName(key: string): string {
        return key.charAt(0).toUpperCase() + key.slice(1);
    }
}

export class HealthTreeItem extends vscode.TreeItem {
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