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
exports.FileWatcher = void 0;
const vscode = __importStar(require("vscode"));
const path = __importStar(require("path"));
class FileWatcher {
    constructor(contextClient, configManager) {
        this.contextClient = contextClient;
        this.configManager = configManager;
        this.fileWatcher = null;
        this.documentChangeListener = null;
        this.activeEditorChangeListener = null;
        this.documentSaveListener = null;
        this.debounceTimers = new Map();
        this.debounceDelay = 500; // 500ms debounce
    }
    startWatching() {
        this.setupFileSystemWatcher();
        this.setupDocumentChangeListener();
        this.setupActiveEditorChangeListener();
        this.setupDocumentSaveListener();
        console.log('[FileWatcher] Started watching for file changes');
    }
    setupFileSystemWatcher() {
        // Watch for file changes in supported languages
        const supportedLanguages = this.configManager.getSupportedLanguages();
        const patterns = supportedLanguages.map(lang => this.getFilePatternForLanguage(lang));
        // Create a pattern that matches all supported file types
        const combinedPattern = `**/*.{${supportedLanguages.map(lang => this.getExtensionForLanguage(lang)).join(',')}}`;
        this.fileWatcher = vscode.workspace.createFileSystemWatcher(combinedPattern);
        // Handle file creation
        this.fileWatcher.onDidCreate(uri => {
            console.log(`[FileWatcher] File created: ${uri.fsPath}`);
            this.handleFileChange(uri.fsPath, 'created');
        });
        // Handle file changes
        this.fileWatcher.onDidChange(uri => {
            console.log(`[FileWatcher] File changed: ${uri.fsPath}`);
            this.handleFileChange(uri.fsPath, 'changed');
        });
        // Handle file deletion
        this.fileWatcher.onDidDelete(uri => {
            console.log(`[FileWatcher] File deleted: ${uri.fsPath}`);
            this.handleFileChange(uri.fsPath, 'deleted');
        });
    }
    setupDocumentChangeListener() {
        this.documentChangeListener = vscode.workspace.onDidChangeTextDocument(event => {
            if (event.document.uri.scheme !== 'file') {
                return;
            }
            const filePath = event.document.uri.fsPath;
            if (!this.shouldWatchFile(filePath)) {
                return;
            }
            // Only process if real-time suggestions are enabled
            if (this.configManager.isRealTimeSuggestionsEnabled()) {
                console.log(`[FileWatcher] Document content changed: ${filePath}`);
                this.handleFileChange(filePath, 'content_changed');
            }
        });
    }
    setupActiveEditorChangeListener() {
        this.activeEditorChangeListener = vscode.window.onDidChangeActiveTextEditor(editor => {
            if (!editor || editor.document.uri.scheme !== 'file') {
                return;
            }
            const filePath = editor.document.uri.fsPath;
            if (!this.shouldWatchFile(filePath)) {
                return;
            }
            console.log(`[FileWatcher] Active editor changed: ${filePath}`);
            this.handleFileChange(filePath, 'opened');
        });
    }
    setupDocumentSaveListener() {
        this.documentSaveListener = vscode.workspace.onDidSaveTextDocument(document => {
            if (document.uri.scheme !== 'file') {
                return;
            }
            const filePath = document.uri.fsPath;
            if (!this.shouldWatchFile(filePath)) {
                return;
            }
            // Only process if auto-analyze on save is enabled
            if (this.configManager.isAutoAnalyzeOnSaveEnabled()) {
                console.log(`[FileWatcher] Document saved: ${filePath}`);
                this.handleFileChange(filePath, 'saved');
            }
        });
    }
    handleFileChange(filePath, changeType) {
        // Clear existing debounce timer for this file
        const existingTimer = this.debounceTimers.get(filePath);
        if (existingTimer) {
            clearTimeout(existingTimer);
        }
        // Set new debounce timer
        const timer = setTimeout(async () => {
            this.debounceTimers.delete(filePath);
            try {
                await this.processFileChange(filePath, changeType);
            }
            catch (error) {
                console.error(`[FileWatcher] Error processing file change for ${filePath}:`, error);
                vscode.window.showErrorMessage(`Failed to process file change: ${error}`);
            }
        }, this.debounceDelay);
        this.debounceTimers.set(filePath, timer);
    }
    async processFileChange(filePath, changeType) {
        console.log(`[FileWatcher] Processing ${changeType} for: ${filePath}`);
        switch (changeType) {
            case 'created':
            case 'changed':
            case 'content_changed':
            case 'saved':
                // Analyze the file for context extraction
                await this.contextClient.analyzeFile(filePath);
                break;
            case 'opened':
                // Send file opened event to get suggestions
                await this.contextClient.analyzeFile(filePath);
                break;
            case 'deleted':
                // Clear suggestions for deleted file
                await this.contextClient.clearSuggestions(filePath);
                break;
        }
        // Emit custom event for other components to listen to
        this.emitFileChangeEvent(filePath, changeType);
    }
    emitFileChangeEvent(filePath, changeType) {
        // Use VS Code's event system to notify other components
        vscode.commands.executeCommand('contextEngine.fileChanged', {
            filePath,
            changeType,
            timestamp: new Date().toISOString()
        });
    }
    shouldWatchFile(filePath) {
        const supportedLanguages = this.configManager.getSupportedLanguages();
        const fileExtension = path.extname(filePath).substring(1); // Remove the dot
        // Check if file extension matches supported languages
        for (const language of supportedLanguages) {
            if (this.getExtensionForLanguage(language) === fileExtension) {
                return true;
            }
        }
        return false;
    }
    getFilePatternForLanguage(language) {
        const extension = this.getExtensionForLanguage(language);
        return `**/*.${extension}`;
    }
    getExtensionForLanguage(language) {
        const extensionMap = {
            'rust': 'rs',
            'typescript': 'ts',
            'javascript': 'js',
            'python': 'py',
            'java': 'java',
            'cpp': 'cpp',
            'csharp': 'cs',
            'c': 'c',
            'go': 'go',
            'php': 'php',
            'ruby': 'rb',
            'swift': 'swift',
            'kotlin': 'kt',
            'scala': 'scala'
        };
        return extensionMap[language] || language;
    }
    dispose() {
        // Clear all debounce timers
        this.debounceTimers.forEach(timer => clearTimeout(timer));
        this.debounceTimers.clear();
        // Dispose of all listeners
        if (this.fileWatcher) {
            this.fileWatcher.dispose();
            this.fileWatcher = null;
        }
        if (this.documentChangeListener) {
            this.documentChangeListener.dispose();
            this.documentChangeListener = null;
        }
        if (this.activeEditorChangeListener) {
            this.activeEditorChangeListener.dispose();
            this.activeEditorChangeListener = null;
        }
        if (this.documentSaveListener) {
            this.documentSaveListener.dispose();
            this.documentSaveListener = null;
        }
        console.log('[FileWatcher] Disposed of all file watchers and listeners');
    }
}
exports.FileWatcher = FileWatcher;
//# sourceMappingURL=fileWatcher.js.map