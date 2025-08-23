import * as vscode from 'vscode';
import { ContextEngineClient } from './contextEngineClient';

export interface TeamMember {
    id: string;
    name: string;
    email: string;
    avatar?: string;
    status: 'online' | 'offline' | 'away';
    lastActivity: Date;
    currentFile?: string;
}

export interface TeamActivity {
    id: string;
    memberId: string;
    memberName: string;
    action: 'created' | 'updated' | 'deleted' | 'analyzed' | 'suggested';
    target: string;
    targetType: 'context' | 'file' | 'suggestion';
    timestamp: Date;
    details?: string;
}

export interface SyncStatus {
    isConnected: boolean;
    lastSync: Date;
    pendingChanges: number;
    conflictCount: number;
}

export class TeamActivityProvider implements vscode.TreeDataProvider<TeamTreeItem> {
    private _onDidChangeTreeData: vscode.EventEmitter<TeamTreeItem | undefined | null | void> = new vscode.EventEmitter<TeamTreeItem | undefined | null | void>();
    readonly onDidChangeTreeData: vscode.Event<TeamTreeItem | undefined | null | void> = this._onDidChangeTreeData.event;

    private teamMembers: TeamMember[] = [];
    private recentActivity: TeamActivity[] = [];
    private syncStatus: SyncStatus = {
        isConnected: false,
        lastSync: new Date(),
        pendingChanges: 0,
        conflictCount: 0
    };

    constructor(private contextClient: ContextEngineClient) {
        // Listen for real-time updates
        this.contextClient.on('teamMemberJoined', (member: TeamMember) => this.handleMemberJoined(member));
        this.contextClient.on('teamMemberLeft', (memberId: string) => this.handleMemberLeft(memberId));
        this.contextClient.on('teamActivity', (activity: TeamActivity) => this.handleTeamActivity(activity));
        this.contextClient.on('syncStatusChanged', (status: SyncStatus) => this.handleSyncStatusChanged(status));
        
        // Initial load
        this.loadTeamData();
        
        // Periodic refresh for activity
        setInterval(() => this.loadRecentActivity(), 30000);
    }

    refresh(): void {
        this.loadTeamData();
        this._onDidChangeTreeData.fire();
    }

    private async loadTeamData(): Promise<void> {
        try {
            const [members, activity, sync] = await Promise.all([
                this.contextClient.getTeamMembers(),
                this.contextClient.getTeamActivity(),
                this.contextClient.getSyncStatus()
            ]);

            this.teamMembers = members || [];
            this.recentActivity = activity || [];
            this.syncStatus = sync || this.syncStatus;
            
            this._onDidChangeTreeData.fire();
        } catch (error) {
            console.error('[TeamActivityProvider] Failed to load team data:', error);
        }
    }

    private async loadRecentActivity(): Promise<void> {
        try {
            const activity = await this.contextClient.getTeamActivity();
            this.recentActivity = activity || [];
            this._onDidChangeTreeData.fire();
        } catch (error) {
            console.error('[TeamActivityProvider] Failed to load recent activity:', error);
        }
    }

    private handleMemberJoined(member: TeamMember): void {
        const existingIndex = this.teamMembers.findIndex(m => m.id === member.id);
        if (existingIndex >= 0) {
            this.teamMembers[existingIndex] = member;
        } else {
            this.teamMembers.push(member);
        }
        this._onDidChangeTreeData.fire();
    }

    private handleMemberLeft(memberId: string): void {
        this.teamMembers = this.teamMembers.filter(m => m.id !== memberId);
        this._onDidChangeTreeData.fire();
    }

    private handleTeamActivity(activity: TeamActivity): void {
        this.recentActivity.unshift(activity);
        // Keep only the last 50 activities
        this.recentActivity = this.recentActivity.slice(0, 50);
        this._onDidChangeTreeData.fire();
    }

    private handleSyncStatusChanged(status: SyncStatus): void {
        this.syncStatus = status;
        this._onDidChangeTreeData.fire();
    }

    getTreeItem(element: TeamTreeItem): vscode.TreeItem {
        return element;
    }

    getChildren(element?: TeamTreeItem): Thenable<TeamTreeItem[]> {
        if (!element) {
            return Promise.resolve(this.getRootItems());
        } else if (element.contextValue === 'membersSection') {
            return Promise.resolve(this.getMemberItems());
        } else if (element.contextValue === 'activitySection') {
            return Promise.resolve(this.getActivityItems());
        } else if (element.contextValue === 'syncSection') {
            return Promise.resolve(this.getSyncItems());
        }
        return Promise.resolve([]);
    }

    private getRootItems(): TeamTreeItem[] {
        const items: TeamTreeItem[] = [];

        // Sync status
        const syncIcon = this.syncStatus.isConnected ? 'check' : 'error';
        const syncColor = this.syncStatus.isConnected ? 
            new vscode.ThemeColor('testing.iconPassed') : 
            new vscode.ThemeColor('errorForeground');
        
        items.push(new TeamTreeItem(
            'Sync Status',
            vscode.TreeItemCollapsibleState.Collapsed,
            'syncSection',
            new vscode.ThemeIcon(syncIcon, syncColor),
            this.syncStatus.isConnected ? 'Connected' : 'Disconnected',
            this.getSyncTooltip()
        ));

        // Team members
        const onlineMembers = this.teamMembers.filter(m => m.status === 'online').length;
        items.push(new TeamTreeItem(
            'Team Members',
            vscode.TreeItemCollapsibleState.Expanded,
            'membersSection',
            new vscode.ThemeIcon('organization'),
            `${onlineMembers}/${this.teamMembers.length} online`,
            `${this.teamMembers.length} team members, ${onlineMembers} currently online`
        ));

        // Recent activity
        items.push(new TeamTreeItem(
            'Recent Activity',
            vscode.TreeItemCollapsibleState.Expanded,
            'activitySection',
            new vscode.ThemeIcon('history'),
            `${this.recentActivity.length} events`,
            'Recent team activity and changes'
        ));

        return items;
    }

    private getMemberItems(): TeamTreeItem[] {
        return this.teamMembers.map(member => {
            const statusIcon = this.getStatusIcon(member.status);
            const statusColor = this.getStatusColor(member.status);
            const timeAgo = this.getTimeAgo(member.lastActivity);
            
            return new TeamTreeItem(
                member.name,
                vscode.TreeItemCollapsibleState.None,
                'teamMember',
                new vscode.ThemeIcon(statusIcon, statusColor),
                `${member.status} • ${timeAgo}`,
                this.getMemberTooltip(member),
                member.currentFile ? {
                    command: 'vscode.open',
                    title: 'Open File',
                    arguments: [vscode.Uri.file(member.currentFile)]
                } : undefined
            );
        });
    }

    private getActivityItems(): TeamTreeItem[] {
        return this.recentActivity.slice(0, 20).map(activity => {
            const actionIcon = this.getActionIcon(activity.action);
            const actionColor = this.getActionColor(activity.action);
            const timeAgo = this.getTimeAgo(activity.timestamp);
            
            return new TeamTreeItem(
                `${activity.memberName} ${activity.action} ${activity.target}`,
                vscode.TreeItemCollapsibleState.None,
                'activity',
                new vscode.ThemeIcon(actionIcon, actionColor),
                timeAgo,
                this.getActivityTooltip(activity),
                {
                    command: 'contextEngine.showActivityDetails',
                    title: 'Show Details',
                    arguments: [activity]
                }
            );
        });
    }

    private getSyncItems(): TeamTreeItem[] {
        const items: TeamTreeItem[] = [];

        // Connection status
        items.push(new TeamTreeItem(
            'Connection',
            vscode.TreeItemCollapsibleState.None,
            'syncConnection',
            new vscode.ThemeIcon(this.syncStatus.isConnected ? 'check' : 'error'),
            this.syncStatus.isConnected ? 'Connected' : 'Disconnected',
            `Real-time sync is ${this.syncStatus.isConnected ? 'active' : 'inactive'}`
        ));

        // Last sync time
        const lastSyncAgo = this.getTimeAgo(this.syncStatus.lastSync);
        items.push(new TeamTreeItem(
            'Last Sync',
            vscode.TreeItemCollapsibleState.None,
            'lastSync',
            new vscode.ThemeIcon('clock'),
            lastSyncAgo,
            `Last synchronization: ${this.syncStatus.lastSync.toLocaleString()}`
        ));

        // Pending changes
        if (this.syncStatus.pendingChanges > 0) {
            items.push(new TeamTreeItem(
                'Pending Changes',
                vscode.TreeItemCollapsibleState.None,
                'pendingChanges',
                new vscode.ThemeIcon('warning', new vscode.ThemeColor('warningForeground')),
                this.syncStatus.pendingChanges.toString(),
                `${this.syncStatus.pendingChanges} changes waiting to be synchronized`,
                {
                    command: 'contextEngine.syncPendingChanges',
                    title: 'Sync Now'
                }
            ));
        }

        // Conflicts
        if (this.syncStatus.conflictCount > 0) {
            items.push(new TeamTreeItem(
                'Conflicts',
                vscode.TreeItemCollapsibleState.None,
                'conflicts',
                new vscode.ThemeIcon('error', new vscode.ThemeColor('errorForeground')),
                this.syncStatus.conflictCount.toString(),
                `${this.syncStatus.conflictCount} conflicts need resolution`,
                {
                    command: 'contextEngine.resolveConflicts',
                    title: 'Resolve Conflicts'
                }
            ));
        }

        return items;
    }

    private getStatusIcon(status: string): string {
        switch (status) {
            case 'online': return 'circle-filled';
            case 'away': return 'circle-outline';
            case 'offline': return 'circle';
            default: return 'question';
        }
    }

    private getStatusColor(status: string): vscode.ThemeColor {
        switch (status) {
            case 'online': return new vscode.ThemeColor('testing.iconPassed');
            case 'away': return new vscode.ThemeColor('warningForeground');
            case 'offline': return new vscode.ThemeColor('disabledForeground');
            default: return new vscode.ThemeColor('foreground');
        }
    }

    private getActionIcon(action: string): string {
        switch (action) {
            case 'created': return 'add';
            case 'updated': return 'edit';
            case 'deleted': return 'trash';
            case 'analyzed': return 'search';
            case 'suggested': return 'lightbulb';
            default: return 'note';
        }
    }

    private getActionColor(action: string): vscode.ThemeColor {
        switch (action) {
            case 'created': return new vscode.ThemeColor('testing.iconPassed');
            case 'updated': return new vscode.ThemeColor('infoForeground');
            case 'deleted': return new vscode.ThemeColor('errorForeground');
            case 'analyzed': return new vscode.ThemeColor('warningForeground');
            case 'suggested': return new vscode.ThemeColor('foreground');
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

    private getMemberTooltip(member: TeamMember): string {
        const lines = [
            `**${member.name}**`,
            `Email: ${member.email}`,
            `Status: ${member.status}`,
            `Last activity: ${member.lastActivity.toLocaleString()}`
        ];

        if (member.currentFile) {
            lines.push(`Current file: ${member.currentFile}`);
        }

        return lines.join('\n');
    }

    private getActivityTooltip(activity: TeamActivity): string {
        const lines = [
            `**${activity.memberName}** ${activity.action} ${activity.target}`,
            `Type: ${activity.targetType}`,
            `Time: ${activity.timestamp.toLocaleString()}`
        ];

        if (activity.details) {
            lines.push(`Details: ${activity.details}`);
        }

        return lines.join('\n');
    }

    private getSyncTooltip(): string {
        const lines = [
            `Connection: ${this.syncStatus.isConnected ? 'Active' : 'Inactive'}`,
            `Last sync: ${this.syncStatus.lastSync.toLocaleString()}`
        ];

        if (this.syncStatus.pendingChanges > 0) {
            lines.push(`Pending changes: ${this.syncStatus.pendingChanges}`);
        }

        if (this.syncStatus.conflictCount > 0) {
            lines.push(`Conflicts: ${this.syncStatus.conflictCount}`);
        }

        return lines.join('\n');
    }
}

export class TeamTreeItem extends vscode.TreeItem {
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