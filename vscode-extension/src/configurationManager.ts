import * as vscode from 'vscode';

export interface AnalysisRule {
    name: string;
    language: string;
    pattern: string;
    contextType: string;
    extractionMethod: string;
    confidence: number;
}

export interface SuggestionTrigger {
    name: string;
    triggerType: string;
    condition: string;
    suggestionTemplate: string;
}

export class ConfigurationManager {
    private static readonly EXTENSION_ID = 'contextEngine';
    
    constructor() {
        // Listen for configuration changes
        vscode.workspace.onDidChangeConfiguration(event => {
            if (event.affectsConfiguration(ConfigurationManager.EXTENSION_ID)) {
                console.log('[ConfigurationManager] Configuration changed');
                this.onConfigurationChanged();
            }
        });
    }

    private onConfigurationChanged(): void {
        // Emit event for other components to react to configuration changes
        vscode.commands.executeCommand('contextEngine.configurationChanged');
    }

    getServerUrl(): string {
        const config = vscode.workspace.getConfiguration(ConfigurationManager.EXTENSION_ID);
        return config.get<string>('serverUrl', 'http://localhost:3000');
    }

    isAutoAnalyzeOnSaveEnabled(): boolean {
        const config = vscode.workspace.getConfiguration(ConfigurationManager.EXTENSION_ID);
        return config.get<boolean>('autoAnalyzeOnSave', true);
    }

    isRealTimeSuggestionsEnabled(): boolean {
        const config = vscode.workspace.getConfiguration(ConfigurationManager.EXTENSION_ID);
        return config.get<boolean>('realTimeSuggestions', true);
    }

    isHoverSuggestionsEnabled(): boolean {
        const config = vscode.workspace.getConfiguration(ConfigurationManager.EXTENSION_ID);
        return config.get<boolean>('enableHoverSuggestions', true);
    }

    isCodeActionsEnabled(): boolean {
        const config = vscode.workspace.getConfiguration(ConfigurationManager.EXTENSION_ID);
        return config.get<boolean>('enableCodeActions', true);
    }

    getSupportedLanguages(): string[] {
        const config = vscode.workspace.getConfiguration(ConfigurationManager.EXTENSION_ID);
        return config.get<string[]>('supportedLanguages', [
            'rust', 'typescript', 'javascript', 'python', 'java', 'cpp', 'csharp'
        ]);
    }

    getAnalysisRules(): AnalysisRule[] {
        const config = vscode.workspace.getConfiguration(ConfigurationManager.EXTENSION_ID);
        const rules = config.get<any[]>('analysisRules', []);
        
        return rules.map(rule => ({
            name: rule.name || 'Unnamed Rule',
            language: rule.language || 'unknown',
            pattern: rule.pattern || '',
            contextType: rule.contextType || 'general',
            extractionMethod: rule.extractionMethod || 'Custom',
            confidence: rule.confidence || 0.5
        }));
    }

    getSuggestionTriggers(): SuggestionTrigger[] {
        const config = vscode.workspace.getConfiguration(ConfigurationManager.EXTENSION_ID);
        const triggers = config.get<any[]>('suggestionTriggers', []);
        
        return triggers.map(trigger => ({
            name: trigger.name || 'Unnamed Trigger',
            triggerType: trigger.triggerType || 'TextChange',
            condition: trigger.condition || '',
            suggestionTemplate: trigger.suggestionTemplate || 'Consider reviewing this code'
        }));
    }

    async updateServerUrl(url: string): Promise<void> {
        const config = vscode.workspace.getConfiguration(ConfigurationManager.EXTENSION_ID);
        await config.update('serverUrl', url, vscode.ConfigurationTarget.Workspace);
    }

    async updateAutoAnalyzeOnSave(enabled: boolean): Promise<void> {
        const config = vscode.workspace.getConfiguration(ConfigurationManager.EXTENSION_ID);
        await config.update('autoAnalyzeOnSave', enabled, vscode.ConfigurationTarget.Workspace);
    }

    async updateRealTimeSuggestions(enabled: boolean): Promise<void> {
        const config = vscode.workspace.getConfiguration(ConfigurationManager.EXTENSION_ID);
        await config.update('realTimeSuggestions', enabled, vscode.ConfigurationTarget.Workspace);
    }

    async updateHoverSuggestions(enabled: boolean): Promise<void> {
        const config = vscode.workspace.getConfiguration(ConfigurationManager.EXTENSION_ID);
        await config.update('enableHoverSuggestions', enabled, vscode.ConfigurationTarget.Workspace);
    }

    async updateCodeActions(enabled: boolean): Promise<void> {
        const config = vscode.workspace.getConfiguration(ConfigurationManager.EXTENSION_ID);
        await config.update('enableCodeActions', enabled, vscode.ConfigurationTarget.Workspace);
    }

    async updateSupportedLanguages(languages: string[]): Promise<void> {
        const config = vscode.workspace.getConfiguration(ConfigurationManager.EXTENSION_ID);
        await config.update('supportedLanguages', languages, vscode.ConfigurationTarget.Workspace);
    }

    async addAnalysisRule(rule: AnalysisRule): Promise<void> {
        const currentRules = this.getAnalysisRules();
        currentRules.push(rule);
        
        const config = vscode.workspace.getConfiguration(ConfigurationManager.EXTENSION_ID);
        await config.update('analysisRules', currentRules, vscode.ConfigurationTarget.Workspace);
    }

    async removeAnalysisRule(ruleName: string): Promise<void> {
        const currentRules = this.getAnalysisRules();
        const filteredRules = currentRules.filter(rule => rule.name !== ruleName);
        
        const config = vscode.workspace.getConfiguration(ConfigurationManager.EXTENSION_ID);
        await config.update('analysisRules', filteredRules, vscode.ConfigurationTarget.Workspace);
    }

    async addSuggestionTrigger(trigger: SuggestionTrigger): Promise<void> {
        const currentTriggers = this.getSuggestionTriggers();
        currentTriggers.push(trigger);
        
        const config = vscode.workspace.getConfiguration(ConfigurationManager.EXTENSION_ID);
        await config.update('suggestionTriggers', currentTriggers, vscode.ConfigurationTarget.Workspace);
    }

    async removeSuggestionTrigger(triggerName: string): Promise<void> {
        const currentTriggers = this.getSuggestionTriggers();
        const filteredTriggers = currentTriggers.filter(trigger => trigger.name !== triggerName);
        
        const config = vscode.workspace.getConfiguration(ConfigurationManager.EXTENSION_ID);
        await config.update('suggestionTriggers', filteredTriggers, vscode.ConfigurationTarget.Workspace);
    }

    getWorkspaceFolder(): string | undefined {
        const workspaceFolders = vscode.workspace.workspaceFolders;
        return workspaceFolders && workspaceFolders.length > 0 
            ? workspaceFolders[0].uri.fsPath 
            : undefined;
    }

    getProjectId(): string {
        return this.getWorkspaceFolder() || 'default';
    }

    // Validation methods
    validateServerUrl(url: string): boolean {
        try {
            new URL(url);
            return true;
        } catch {
            return false;
        }
    }

    validateAnalysisRule(rule: AnalysisRule): string[] {
        const errors: string[] = [];
        
        if (!rule.name || rule.name.trim() === '') {
            errors.push('Rule name is required');
        }
        
        if (!rule.language || rule.language.trim() === '') {
            errors.push('Language is required');
        }
        
        if (!rule.pattern || rule.pattern.trim() === '') {
            errors.push('Pattern is required');
        } else {
            try {
                new RegExp(rule.pattern);
            } catch {
                errors.push('Pattern is not a valid regular expression');
            }
        }
        
        if (!rule.contextType || rule.contextType.trim() === '') {
            errors.push('Context type is required');
        }
        
        if (rule.confidence < 0 || rule.confidence > 1) {
            errors.push('Confidence must be between 0 and 1');
        }
        
        return errors;
    }

    validateSuggestionTrigger(trigger: SuggestionTrigger): string[] {
        const errors: string[] = [];
        
        if (!trigger.name || trigger.name.trim() === '') {
            errors.push('Trigger name is required');
        }
        
        if (!trigger.triggerType || trigger.triggerType.trim() === '') {
            errors.push('Trigger type is required');
        }
        
        if (!trigger.condition || trigger.condition.trim() === '') {
            errors.push('Condition is required');
        }
        
        if (!trigger.suggestionTemplate || trigger.suggestionTemplate.trim() === '') {
            errors.push('Suggestion template is required');
        }
        
        return errors;
    }

    // Export/Import configuration
    exportConfiguration(): any {
        const config = vscode.workspace.getConfiguration(ConfigurationManager.EXTENSION_ID);
        return {
            serverUrl: this.getServerUrl(),
            autoAnalyzeOnSave: this.isAutoAnalyzeOnSaveEnabled(),
            realTimeSuggestions: this.isRealTimeSuggestionsEnabled(),
            enableHoverSuggestions: this.isHoverSuggestionsEnabled(),
            enableCodeActions: this.isCodeActionsEnabled(),
            supportedLanguages: this.getSupportedLanguages(),
            analysisRules: this.getAnalysisRules(),
            suggestionTriggers: this.getSuggestionTriggers()
        };
    }

    async importConfiguration(configData: any): Promise<void> {
        const config = vscode.workspace.getConfiguration(ConfigurationManager.EXTENSION_ID);
        
        if (configData.serverUrl) {
            await config.update('serverUrl', configData.serverUrl, vscode.ConfigurationTarget.Workspace);
        }
        
        if (typeof configData.autoAnalyzeOnSave === 'boolean') {
            await config.update('autoAnalyzeOnSave', configData.autoAnalyzeOnSave, vscode.ConfigurationTarget.Workspace);
        }
        
        if (typeof configData.realTimeSuggestions === 'boolean') {
            await config.update('realTimeSuggestions', configData.realTimeSuggestions, vscode.ConfigurationTarget.Workspace);
        }
        
        if (typeof configData.enableHoverSuggestions === 'boolean') {
            await config.update('enableHoverSuggestions', configData.enableHoverSuggestions, vscode.ConfigurationTarget.Workspace);
        }
        
        if (typeof configData.enableCodeActions === 'boolean') {
            await config.update('enableCodeActions', configData.enableCodeActions, vscode.ConfigurationTarget.Workspace);
        }
        
        if (Array.isArray(configData.supportedLanguages)) {
            await config.update('supportedLanguages', configData.supportedLanguages, vscode.ConfigurationTarget.Workspace);
        }
        
        if (Array.isArray(configData.analysisRules)) {
            await config.update('analysisRules', configData.analysisRules, vscode.ConfigurationTarget.Workspace);
        }
        
        if (Array.isArray(configData.suggestionTriggers)) {
            await config.update('suggestionTriggers', configData.suggestionTriggers, vscode.ConfigurationTarget.Workspace);
        }
    }
}