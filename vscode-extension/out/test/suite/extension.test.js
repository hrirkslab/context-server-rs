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
const assert = __importStar(require("assert"));
const vscode = __importStar(require("vscode"));
const configurationManager_1 = require("../../configurationManager");
suite('Extension Test Suite', () => {
    vscode.window.showInformationMessage('Start all tests.');
    test('Configuration Manager', () => {
        const configManager = new configurationManager_1.ConfigurationManager();
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
        const configManager = new configurationManager_1.ConfigurationManager();
        assert.strictEqual(configManager.validateServerUrl('http://localhost:3000'), true);
        assert.strictEqual(configManager.validateServerUrl('https://example.com'), true);
        assert.strictEqual(configManager.validateServerUrl('invalid-url'), false);
        assert.strictEqual(configManager.validateServerUrl(''), false);
    });
    test('Analysis Rule Validation', () => {
        const configManager = new configurationManager_1.ConfigurationManager();
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
//# sourceMappingURL=extension.test.js.map