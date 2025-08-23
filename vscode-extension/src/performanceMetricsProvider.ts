import * as vscode from 'vscode';
import { ContextEngineClient } from './contextEngineClient';

export interface PerformanceMetrics {
    responseTime: {
        average: number;
        p95: number;
        p99: number;
        trend: 'improving' | 'degrading' | 'stable';
    };
    throughput: {
        requestsPerSecond: number;
        contextQueriesPerMinute: number;
        analysisPerHour: number;
    };
    resourceUsage: {
        memoryUsage: number;
        cpuUsage: number;
        diskUsage: number;
        networkUsage: number;
    };
    cacheMetrics: {
        hitRate: number;
        missRate: number;
        evictionRate: number;
        size: number;
    };
    errorMetrics: {
        errorRate: number;
        timeoutRate: number;
        retryRate: number;
        recentErrors: ErrorInfo[];
    };
    usagePatterns: {
        peakHours: number[];
        mostUsedFeatures: FeatureUsage[];
        userActivity: UserActivity[];
    };
}

export interface ErrorInfo {
    timestamp: Date;
    type: string;
    message: string;
    count: number;
}

export interface FeatureUsage {
    feature: string;
    usage: number;
    trend: 'up' | 'down' | 'stable';
}

export interface UserActivity {
    hour: number;
    requests: number;
    users: number;
}

export class PerformanceMetricsProvider implements vscode.TreeDataProvider<MetricsTreeItem> {
    private _onDidChangeTreeData: vscode.EventEmitter<MetricsTreeItem | undefined | null | void> = new vscode.EventEmitter<MetricsTreeItem | undefined | null | void>();
    readonly onDidChangeTreeData: vscode.Event<MetricsTreeItem | undefined | null | void> = this._onDidChangeTreeData.event;

    private metrics: PerformanceMetrics | null = null;
    private refreshInterval: NodeJS.Timeout | null = null;

    constructor(private contextClient: ContextEngineClient) {
        // Initial load and periodic refresh
        this.loadMetrics();
        this.startPeriodicRefresh();
    }

    refresh(): void {
        this.loadMetrics();
        this._onDidChangeTreeData.fire();
    }

    dispose(): void {
        if (this.refreshInterval) {
            clearInterval(this.refreshInterval);
            this.refreshInterval = null;
        }
    }

    private startPeriodicRefresh(): void {
        // Refresh metrics every 15 seconds
        this.refreshInterval = setInterval(() => {
            this.loadMetrics();
        }, 15000);
    }

    private async loadMetrics(): Promise<void> {
        try {
            const metrics = await this.contextClient.getPerformanceMetrics();
            this.metrics = metrics || this.getDefaultMetrics();
            this._onDidChangeTreeData.fire();
        } catch (error) {
            console.error('[PerformanceMetricsProvider] Failed to load metrics:', error);
            this.metrics = this.getDefaultMetrics();
        }
    }

    private getDefaultMetrics(): PerformanceMetrics {
        return {
            responseTime: { average: 0, p95: 0, p99: 0, trend: 'stable' },
            throughput: { requestsPerSecond: 0, contextQueriesPerMinute: 0, analysisPerHour: 0 },
            resourceUsage: { memoryUsage: 0, cpuUsage: 0, diskUsage: 0, networkUsage: 0 },
            cacheMetrics: { hitRate: 0, missRate: 0, evictionRate: 0, size: 0 },
            errorMetrics: { errorRate: 0, timeoutRate: 0, retryRate: 0, recentErrors: [] },
            usagePatterns: { peakHours: [], mostUsedFeatures: [], userActivity: [] }
        };
    }

    getTreeItem(element: MetricsTreeItem): vscode.TreeItem {
        return element;
    }

    getChildren(element?: MetricsTreeItem): Thenable<MetricsTreeItem[]> {
        if (!element) {
            return Promise.resolve(this.getRootItems());
        } else if (element.contextValue === 'responseTimeSection') {
            return Promise.resolve(this.getResponseTimeItems());
        } else if (element.contextValue === 'throughputSection') {
            return Promise.resolve(this.getThroughputItems());
        } else if (element.contextValue === 'resourceSection') {
            return Promise.resolve(this.getResourceItems());
        } else if (element.contextValue === 'cacheSection') {
            return Promise.resolve(this.getCacheItems());
        } else if (element.contextValue === 'errorSection') {
            return Promise.resolve(this.getErrorItems());
        } else if (element.contextValue === 'usageSection') {
            return Promise.resolve(this.getUsageItems());
        }
        return Promise.resolve([]);
    }

    private getRootItems(): MetricsTreeItem[] {
        if (!this.metrics) {
            return [new MetricsTreeItem(
                'Loading performance metrics...',
                vscode.TreeItemCollapsibleState.None,
                'loading',
                new vscode.ThemeIcon('loading~spin')
            )];
        }

        const items: MetricsTreeItem[] = [];

        // Response Time
        const responseIcon = this.getTrendIcon(this.metrics.responseTime.trend);
        const responseColor = this.getTrendColor(this.metrics.responseTime.trend);
        items.push(new MetricsTreeItem(
            'Response Time',
            vscode.TreeItemCollapsibleState.Collapsed,
            'responseTimeSection',
            new vscode.ThemeIcon(responseIcon, responseColor),
            `${this.metrics.responseTime.average.toFixed(0)}ms avg`,
            `Average response time with trend: ${this.metrics.responseTime.trend}`
        ));

        // Throughput
        items.push(new MetricsTreeItem(
            'Throughput',
            vscode.TreeItemCollapsibleState.Collapsed,
            'throughputSection',
            new vscode.ThemeIcon('dashboard'),
            `${this.metrics.throughput.requestsPerSecond.toFixed(1)} req/s`,
            'Request throughput and processing rates'
        ));

        // Resource Usage
        const maxResource = Math.max(
            this.metrics.resourceUsage.memoryUsage,
            this.metrics.resourceUsage.cpuUsage
        );
        const resourceIcon = maxResource > 80 ? 'warning' : maxResource > 60 ? 'info' : 'check';
        const resourceColor = maxResource > 80 ? 
            new vscode.ThemeColor('warningForeground') : 
            maxResource > 60 ? 
                new vscode.ThemeColor('infoForeground') : 
                new vscode.ThemeColor('testing.iconPassed');

        items.push(new MetricsTreeItem(
            'Resource Usage',
            vscode.TreeItemCollapsibleState.Collapsed,
            'resourceSection',
            new vscode.ThemeIcon(resourceIcon, resourceColor),
            `${maxResource.toFixed(0)}% peak`,
            'System resource utilization'
        ));

        // Cache Performance
        const cacheIcon = this.metrics.cacheMetrics.hitRate > 80 ? 'check' : 
                         this.metrics.cacheMetrics.hitRate > 60 ? 'info' : 'warning';
        const cacheColor = this.metrics.cacheMetrics.hitRate > 80 ? 
            new vscode.ThemeColor('testing.iconPassed') : 
            this.metrics.cacheMetrics.hitRate > 60 ? 
                new vscode.ThemeColor('infoForeground') : 
                new vscode.ThemeColor('warningForeground');

        items.push(new MetricsTreeItem(
            'Cache Performance',
            vscode.TreeItemCollapsibleState.Collapsed,
            'cacheSection',
            new vscode.ThemeIcon(cacheIcon, cacheColor),
            `${this.metrics.cacheMetrics.hitRate.toFixed(1)}% hit rate`,
            'Cache hit rates and efficiency'
        ));

        // Error Metrics
        if (this.metrics.errorMetrics.errorRate > 0 || this.metrics.errorMetrics.recentErrors.length > 0) {
            const errorIcon = this.metrics.errorMetrics.errorRate > 5 ? 'error' : 
                             this.metrics.errorMetrics.errorRate > 1 ? 'warning' : 'info';
            const errorColor = this.metrics.errorMetrics.errorRate > 5 ? 
                new vscode.ThemeColor('errorForeground') : 
                this.metrics.errorMetrics.errorRate > 1 ? 
                    new vscode.ThemeColor('warningForeground') : 
                    new vscode.ThemeColor('infoForeground');

            items.push(new MetricsTreeItem(
                'Error Metrics',
                vscode.TreeItemCollapsibleState.Collapsed,
                'errorSection',
                new vscode.ThemeIcon(errorIcon, errorColor),
                `${this.metrics.errorMetrics.errorRate.toFixed(2)}% error rate`,
                'Error rates and recent issues'
            ));
        }

        // Usage Patterns
        items.push(new MetricsTreeItem(
            'Usage Patterns',
            vscode.TreeItemCollapsibleState.Collapsed,
            'usageSection',
            new vscode.ThemeIcon('graph'),
            `${this.metrics.usagePatterns.mostUsedFeatures.length} features`,
            'Usage patterns and feature adoption'
        ));

        return items;
    }

    private getResponseTimeItems(): MetricsTreeItem[] {
        if (!this.metrics) return [];

        return [
            new MetricsTreeItem(
                'Average',
                vscode.TreeItemCollapsibleState.None,
                'metric',
                new vscode.ThemeIcon('clock'),
                `${this.metrics.responseTime.average.toFixed(0)}ms`,
                'Average response time across all requests'
            ),
            new MetricsTreeItem(
                '95th Percentile',
                vscode.TreeItemCollapsibleState.None,
                'metric',
                new vscode.ThemeIcon('clock'),
                `${this.metrics.responseTime.p95.toFixed(0)}ms`,
                '95% of requests complete within this time'
            ),
            new MetricsTreeItem(
                '99th Percentile',
                vscode.TreeItemCollapsibleState.None,
                'metric',
                new vscode.ThemeIcon('clock'),
                `${this.metrics.responseTime.p99.toFixed(0)}ms`,
                '99% of requests complete within this time'
            ),
            new MetricsTreeItem(
                'Trend',
                vscode.TreeItemCollapsibleState.None,
                'trend',
                new vscode.ThemeIcon(this.getTrendIcon(this.metrics.responseTime.trend)),
                this.metrics.responseTime.trend,
                `Response time trend: ${this.metrics.responseTime.trend}`
            )
        ];
    }

    private getThroughputItems(): MetricsTreeItem[] {
        if (!this.metrics) return [];

        return [
            new MetricsTreeItem(
                'Requests per Second',
                vscode.TreeItemCollapsibleState.None,
                'metric',
                new vscode.ThemeIcon('pulse'),
                `${this.metrics.throughput.requestsPerSecond.toFixed(1)}`,
                'HTTP requests processed per second'
            ),
            new MetricsTreeItem(
                'Context Queries per Minute',
                vscode.TreeItemCollapsibleState.None,
                'metric',
                new vscode.ThemeIcon('search'),
                `${this.metrics.throughput.contextQueriesPerMinute.toFixed(0)}`,
                'Context queries processed per minute'
            ),
            new MetricsTreeItem(
                'Analysis per Hour',
                vscode.TreeItemCollapsibleState.None,
                'metric',
                new vscode.ThemeIcon('beaker'),
                `${this.metrics.throughput.analysisPerHour.toFixed(0)}`,
                'File analyses completed per hour'
            )
        ];
    }

    private getResourceItems(): MetricsTreeItem[] {
        if (!this.metrics) return [];

        return [
            new MetricsTreeItem(
                'Memory Usage',
                vscode.TreeItemCollapsibleState.None,
                'resource',
                new vscode.ThemeIcon('server'),
                `${this.metrics.resourceUsage.memoryUsage.toFixed(1)}%`,
                'Current memory utilization'
            ),
            new MetricsTreeItem(
                'CPU Usage',
                vscode.TreeItemCollapsibleState.None,
                'resource',
                new vscode.ThemeIcon('cpu'),
                `${this.metrics.resourceUsage.cpuUsage.toFixed(1)}%`,
                'Current CPU utilization'
            ),
            new MetricsTreeItem(
                'Disk Usage',
                vscode.TreeItemCollapsibleState.None,
                'resource',
                new vscode.ThemeIcon('database'),
                `${this.metrics.resourceUsage.diskUsage.toFixed(1)}%`,
                'Current disk space utilization'
            ),
            new MetricsTreeItem(
                'Network Usage',
                vscode.TreeItemCollapsibleState.None,
                'resource',
                new vscode.ThemeIcon('globe'),
                `${this.metrics.resourceUsage.networkUsage.toFixed(1)} MB/s`,
                'Current network throughput'
            )
        ];
    }

    private getCacheItems(): MetricsTreeItem[] {
        if (!this.metrics) return [];

        return [
            new MetricsTreeItem(
                'Hit Rate',
                vscode.TreeItemCollapsibleState.None,
                'cache',
                new vscode.ThemeIcon('check'),
                `${this.metrics.cacheMetrics.hitRate.toFixed(1)}%`,
                'Percentage of requests served from cache'
            ),
            new MetricsTreeItem(
                'Miss Rate',
                vscode.TreeItemCollapsibleState.None,
                'cache',
                new vscode.ThemeIcon('x'),
                `${this.metrics.cacheMetrics.missRate.toFixed(1)}%`,
                'Percentage of requests not found in cache'
            ),
            new MetricsTreeItem(
                'Eviction Rate',
                vscode.TreeItemCollapsibleState.None,
                'cache',
                new vscode.ThemeIcon('trash'),
                `${this.metrics.cacheMetrics.evictionRate.toFixed(1)}/min`,
                'Cache entries evicted per minute'
            ),
            new MetricsTreeItem(
                'Cache Size',
                vscode.TreeItemCollapsibleState.None,
                'cache',
                new vscode.ThemeIcon('package'),
                `${this.formatBytes(this.metrics.cacheMetrics.size)}`,
                'Current cache size in memory'
            )
        ];
    }

    private getErrorItems(): MetricsTreeItem[] {
        if (!this.metrics) return [];

        const items = [
            new MetricsTreeItem(
                'Error Rate',
                vscode.TreeItemCollapsibleState.None,
                'error',
                new vscode.ThemeIcon('error'),
                `${this.metrics.errorMetrics.errorRate.toFixed(2)}%`,
                'Percentage of requests that resulted in errors'
            ),
            new MetricsTreeItem(
                'Timeout Rate',
                vscode.TreeItemCollapsibleState.None,
                'error',
                new vscode.ThemeIcon('clock'),
                `${this.metrics.errorMetrics.timeoutRate.toFixed(2)}%`,
                'Percentage of requests that timed out'
            ),
            new MetricsTreeItem(
                'Retry Rate',
                vscode.TreeItemCollapsibleState.None,
                'error',
                new vscode.ThemeIcon('refresh'),
                `${this.metrics.errorMetrics.retryRate.toFixed(2)}%`,
                'Percentage of requests that were retried'
            )
        ];

        // Add recent errors
        if (this.metrics.errorMetrics.recentErrors.length > 0) {
            items.push(...this.metrics.errorMetrics.recentErrors.slice(0, 5).map(error => 
                new MetricsTreeItem(
                    error.type,
                    vscode.TreeItemCollapsibleState.None,
                    'recentError',
                    new vscode.ThemeIcon('warning'),
                    `${error.count}x`,
                    `${error.message}\nOccurred: ${error.timestamp.toLocaleString()}\nCount: ${error.count}`,
                    {
                        command: 'contextEngine.showErrorDetails',
                        title: 'Show Details',
                        arguments: [error]
                    }
                )
            ));
        }

        return items;
    }

    private getUsageItems(): MetricsTreeItem[] {
        if (!this.metrics) return [];

        const items: MetricsTreeItem[] = [];

        // Peak hours
        if (this.metrics.usagePatterns.peakHours.length > 0) {
            const peakHoursStr = this.metrics.usagePatterns.peakHours
                .map(h => `${h}:00`)
                .join(', ');
            items.push(new MetricsTreeItem(
                'Peak Hours',
                vscode.TreeItemCollapsibleState.None,
                'usage',
                new vscode.ThemeIcon('clock'),
                peakHoursStr,
                `Highest usage occurs at: ${peakHoursStr}`
            ));
        }

        // Most used features
        this.metrics.usagePatterns.mostUsedFeatures.slice(0, 5).forEach(feature => {
            const trendIcon = this.getTrendIcon(feature.trend);
            items.push(new MetricsTreeItem(
                feature.feature,
                vscode.TreeItemCollapsibleState.None,
                'feature',
                new vscode.ThemeIcon(trendIcon),
                `${feature.usage} uses`,
                `Feature usage: ${feature.usage} times, trend: ${feature.trend}`
            ));
        });

        return items;
    }

    private getTrendIcon(trend: string): string {
        switch (trend) {
            case 'improving':
            case 'up': return 'trending-up';
            case 'degrading':
            case 'down': return 'trending-down';
            case 'stable': return 'arrow-right';
            default: return 'question';
        }
    }

    private getTrendColor(trend: string): vscode.ThemeColor {
        switch (trend) {
            case 'improving':
            case 'up': return new vscode.ThemeColor('testing.iconPassed');
            case 'degrading':
            case 'down': return new vscode.ThemeColor('errorForeground');
            case 'stable': return new vscode.ThemeColor('infoForeground');
            default: return new vscode.ThemeColor('foreground');
        }
    }

    private formatBytes(bytes: number): string {
        if (bytes === 0) return '0 B';
        const k = 1024;
        const sizes = ['B', 'KB', 'MB', 'GB'];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
    }
}

export class MetricsTreeItem extends vscode.TreeItem {
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