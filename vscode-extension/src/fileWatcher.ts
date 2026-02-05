import * as vscode from 'vscode';
import * as path from 'path';
import { ContextEngineClient } from './contextEngineClient';
import { ConfigurationManager } from './configurationManager';

export class FileWatcher {
    private fileWatcher: vscode.FileSystemWatcher | null = null;
    private documentChangeListener: vscode.Disposable | null = null;
    private activeEditorChangeListener: vscode.Disposable | null = null;
    private documentSaveListener: vscode.Disposable | null = null;
    private debounceTimers: Map<string, NodeJS.Timeout> = new Map();
    private readonly debounceDelay = 500; // 500ms debounce

    constructor(
        private contextClient: ContextEngineClient,
        private configManager: ConfigurationManager
    ) {}

    startWatching(): void {
        this.setupFileSystemWatcher();
        this.setupDocumentChangeListener();
        this.setupActiveEditorChangeListener();
        this.setupDocumentSaveListener();
        
        console.log('[FileWatcher] Started watching for file changes');
    }

    private setupFileSystemWatcher(): void {
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

    private setupDocumentChangeListener(): void {
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

    private setupActiveEditorChangeListener(): void {
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

    private setupDocumentSaveListener(): void {
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

    private handleFileChange(filePath: string, changeType: string): void {
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
            } catch (error) {
                console.error(`[FileWatcher] Error processing file change for ${filePath}:`, error);
                vscode.window.showErrorMessage(`Failed to process file change: ${error}`);
            }
        }, this.debounceDelay);

        this.debounceTimers.set(filePath, timer);
    }

    private async processFileChange(filePath: string, changeType: string): Promise<void> {
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

    private emitFileChangeEvent(filePath: string, changeType: string): void {
        // Use VS Code's event system to notify other components
        vscode.commands.executeCommand('contextEngine.fileChanged', {
            filePath,
            changeType,
            timestamp: new Date().toISOString()
        });
    }

    private shouldWatchFile(filePath: string): boolean {
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

    private getFilePatternForLanguage(language: string): string {
        const extension = this.getExtensionForLanguage(language);
        return `**/*.${extension}`;
    }

    private getExtensionForLanguage(language: string): string {
        const extensionMap: Record<string, string> = {
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

    dispose(): void {
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