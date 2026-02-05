import * as assert from 'assert';
import * as vscode from 'vscode';
import { ConfigurationManager } from '../../configurationManager';

suite('Extension Test Suite', () => {
    vscode.window.showInformationMessage('Start all tests.');

    test('Configuration Manager', () => {
        const configManager = new ConfigurationManager();
        
        // Test default values
        assert.strictEqual(configManager.getServerUrl(), 'http://localhost:3000');
        assert.strictEqual(configManager.isAutoAnalyzeOnSaveEnabled(), true);
        assert.strictEqual(configManager.isRealTimeSuggestionsEnabled(), true);
        
        // Test supported languages
        const languages = configManager.getSupportedLanguages();
        assert.ok(languages.includes('rust'));
        assert.ok(languages.includes('typescript'));
        assert.ok(languages.includes('javascript'));
    });

    test('Server URL Validation', () => {
        const configManager = new ConfigurationManager();
        
        assert.strictEqual(configManager.validateServerUrl('http://localhost:3000'), true);
        assert.strictEqual(configManager.validateServerUrl('https://example.com'), true);
        assert.strictEqual(configManager.validateServerUrl('invalid-url'), false);
        assert.strictEqual(configManager.validateServerUrl(''), false);
    });

    test('Analysis Rule Validation', () => {
        const configManager = new ConfigurationManager();
        
        const validRule = {
            name: 'Test Rule',
            language: 'rust',
            pattern: 'TODO:\\s*(.+)',
            contextType: 'general',
            extractionMethod: 'RegexCapture',
            confidence: 0.8
        };
        
        const errors = configManager.validateAnalysisRule(validRule);
        assert.strictEqual(errors.length, 0);
        
        const invalidRule = {
            name: '',
            language: '',
            pattern: '[invalid-regex',
            contextType: '',
            extractionMethod: 'RegexCapture',
            confidence: 1.5
        };
        
        const invalidErrors = configManager.validateAnalysisRule(invalidRule);
        assert.ok(invalidErrors.length > 0);
    });
});