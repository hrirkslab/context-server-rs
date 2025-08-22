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
exports.ConfigurationManager = void 0;
const vscode = __importStar(require("vscode"));
class ConfigurationManager {
    constructor() {
        // Listen for configuration changes
        vscode.workspace.onDidChangeConfiguration(event => {
            if (event.affectsConfiguration(ConfigurationManager.EXTENSION_ID)) {
                console.log('[ConfigurationManager] Configuration changed');
                this.onConfigurationChanged();
            }
        });
    }
    onConfigurationChanged() {
        // Emit event for other components to react to configuration changes
        vscode.commands.executeCommand('contextEngine.configurationChanged');
    }
    getServerUrl() {
        const config = vscode.workspace.getConfiguration(ConfigurationManager.EXTENSION_ID);
        return config.get('serverUrl', 'http://localhost:3000');
    }
    isAutoAnalyzeOnSaveEnabled() {
        const config = vscode.workspace.getConfiguration(ConfigurationManager.EXTENSION_ID);
        return config.get('autoAnalyzeOnSave', true);
    }
    isRealTimeSuggestionsEnabled() {
        const config = vscode.workspace.getConfiguration(ConfigurationManager.EXTENSION_ID);
        return config.get('realTimeSuggestions', true);
    }
    isHoverSuggestionsEnabled() {
        const config = vscode.workspace.getConfiguration(ConfigurationManager.EXTENSION_ID);
        return config.get('enableHoverSuggestions', true);
    }
    isCodeActionsEnabled() {
        const config = vscode.workspace.getConfiguration(ConfigurationManager.EXTENSION_ID);
        return config.get('enableCodeActions', true);
    }
    getSupportedLanguages() {
        const config = vscode.workspace.getConfiguration(ConfigurationManager.EXTENSION_ID);
        return config.get('supportedLanguages', [
            'rust', 'typescript', 'javascript', 'python', 'java', 'cpp', 'csharp'
        ]);
    }
    getAnalysisRules() {
        const config = vscode.workspace.getConfiguration(ConfigurationManager.EXTENSION_ID);
        const rules = config.get('analysisRules', []);
        return rules.map(rule => ({
            name: rule.name || 'Unnamed Rule',
            language: rule.language || 'unknown',
            pattern: rule.pattern || '',
            contextType: rule.contextType || 'general',
            extractionMethod: rule.extractionMethod || 'Custom',
            confidence: rule.confidence || 0.5
        }));
    }
    getSuggestionTriggers() {
        const config = vscode.workspace.getConfiguration(ConfigurationManager.EXTENSION_ID);
        const triggers = config.get('suggestionTriggers', []);
        return triggers.map(trigger => ({
            name: trigger.name || 'Unnamed Trigger',
            triggerType: trigger.triggerType || 'TextChange',
            condition: trigger.condition || '',
            suggestionTemplate: trigger.suggestionTemplate || 'Consider reviewing this code'
        }));
    }
    async updateServerUrl(url) {
        const config = vscode.workspace.getConfiguration(ConfigurationManager.EXTENSION_ID);
        await config.update('serverUrl', url, vscode.ConfigurationTarget.Workspace);
    }
    async updateAutoAnalyzeOnSave(enabled) {
        const config = vscode.workspace.getConfiguration(ConfigurationManager.EXTENSION_ID);
        await config.update('autoAnalyzeOnSave', enabled, vscode.ConfigurationTarget.Workspace);
    }
    async updateRealTimeSuggestions(enabled) {
        const config = vscode.workspace.getConfiguration(ConfigurationManager.EXTENSION_ID);
        await config.update('realTimeSuggestions', enabled, vscode.ConfigurationTarget.Workspace);
    }
    async updateHoverSuggestions(enabled) {
        const config = vscode.workspace.getConfiguration(ConfigurationManager.EXTENSION_ID);
        await config.update('enableHoverSuggestions', enabled, vscode.ConfigurationTarget.Workspace);
    }
    async updateCodeActions(enabled) {
        const config = vscode.workspace.getConfiguration(ConfigurationManager.EXTENSION_ID);
        await config.update('enableCodeActions', enabled, vscode.ConfigurationTarget.Workspace);
    }
    async updateSupportedLanguages(languages) {
        const config = vscode.workspace.getConfiguration(ConfigurationManager.EXTENSION_ID);
        await config.update('supportedLanguages', languages, vscode.ConfigurationTarget.Workspace);
    }
    async addAnalysisRule(rule) {
        const currentRules = this.getAnalysisRules();
        currentRules.push(rule);
        const config = vscode.workspace.getConfiguration(ConfigurationManager.EXTENSION_ID);
        await config.update('analysisRules', currentRules, vscode.ConfigurationTarget.Workspace);
    }
    async removeAnalysisRule(ruleName) {
        const currentRules = this.getAnalysisRules();
        const filteredRules = currentRules.filter(rule => rule.name !== ruleName);
        const config = vscode.workspace.getConfiguration(ConfigurationManager.EXTENSION_ID);
        await config.update('analysisRules', filteredRules, vscode.ConfigurationTarget.Workspace);
    }
    async addSuggestionTrigger(trigger) {
        const currentTriggers = this.getSuggestionTriggers();
        currentTriggers.push(trigger);
        const config = vscode.workspace.getConfiguration(ConfigurationManager.EXTENSION_ID);
        await config.update('suggestionTriggers', currentTriggers, vscode.ConfigurationTarget.Workspace);
    }
    async removeSuggestionTrigger(triggerName) {
        const currentTriggers = this.getSuggestionTriggers();
        const filteredTriggers = currentTriggers.filter(trigger => trigger.name !== triggerName);
        const config = vscode.workspace.getConfiguration(ConfigurationManager.EXTENSION_ID);
        await config.update('suggestionTriggers', filteredTriggers, vscode.ConfigurationTarget.Workspace);
    }
    getWorkspaceFolder() {
        const workspaceFolders = vscode.workspace.workspaceFolders;
        return workspaceFolders && workspaceFolders.length > 0
            ? workspaceFolders[0].uri.fsPath
            : undefined;
    }
    getProjectId() {
        return this.getWorkspaceFolder() || 'default';
    }
    // Validation methods
    validateServerUrl(url) {
        try {
            new URL(url);
            return true;
        }
        catch {
            return false;
        }
    }
    validateAnalysisRule(rule) {
        const errors = [];
        if (!rule.name || rule.name.trim() === '') {
            errors.push('Rule name is required');
        }
        if (!rule.language || rule.language.trim() === '') {
            errors.push('Language is required');
        }
        if (!rule.pattern || rule.pattern.trim() === '') {
            errors.push('Pattern is required');
        }
        else {
            try {
                new RegExp(rule.pattern);
            }
            catch {
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
    validateSuggestionTrigger(trigger) {
        const errors = [];
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
    exportConfiguration() {
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
    async importConfiguration(configData) {
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
exports.ConfigurationManager = ConfigurationManager;
ConfigurationManager.EXTENSION_ID = 'contextEngine';
//# sourceMappingURL=configurationManager.js.map