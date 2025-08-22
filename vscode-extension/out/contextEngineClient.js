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
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.ContextEngineClient = void 0;
const vscode = __importStar(require("vscode"));
const axios_1 = __importDefault(require("axios"));
const ws_1 = __importDefault(require("ws"));
class ContextEngineClient {
    constructor(configManager) {
        this.configManager = configManager;
        this.websocket = null;
        this.reconnectAttempts = 0;
        this.maxReconnectAttempts = 5;
        this.reconnectDelay = 1000;
        this.eventHandlers = new Map();
        const serverUrl = this.configManager.getServerUrl();
        this.httpClient = axios_1.default.create({
            baseURL: serverUrl,
            timeout: 10000,
            headers: {
                'Content-Type': 'application/json',
                'User-Agent': 'VSCode-ContextEngine/1.0.0'
            }
        });
        // Setup request/response interceptors for logging
        this.httpClient.interceptors.request.use((config) => {
            console.log(`[ContextEngine] HTTP Request: ${config.method?.toUpperCase()} ${config.url}`);
            return config;
        }, (error) => {
            console.error('[ContextEngine] HTTP Request Error:', error);
            return Promise.reject(error);
        });
        this.httpClient.interceptors.response.use((response) => {
            console.log(`[ContextEngine] HTTP Response: ${response.status} ${response.config.url}`);
            return response;
        }, (error) => {
            console.error('[ContextEngine] HTTP Response Error:', error.response?.status, error.message);
            return Promise.reject(error);
        });
    }
    async connect() {
        try {
            // Test HTTP connection first
            await this.httpClient.get('/health');
            console.log('[ContextEngine] HTTP connection established');
            // Establish WebSocket connection for real-time updates
            await this.connectWebSocket();
        }
        catch (error) {
            console.error('[ContextEngine] Failed to connect:', error);
            throw new Error(`Failed to connect to Context Engine server: ${error}`);
        }
    }
    async connectWebSocket() {
        const serverUrl = this.configManager.getServerUrl();
        const wsUrl = serverUrl.replace(/^http/, 'ws') + '/ws';
        return new Promise((resolve, reject) => {
            try {
                this.websocket = new ws_1.default(wsUrl);
                this.websocket.on('open', () => {
                    console.log('[ContextEngine] WebSocket connection established');
                    this.reconnectAttempts = 0;
                    // Send initial subscription message
                    this.sendWebSocketMessage({
                        type: 'subscribe',
                        filters: {
                            project_id: vscode.workspace.workspaceFolders?.[0]?.uri.fsPath || 'default'
                        }
                    });
                    resolve();
                });
                this.websocket.on('message', (data) => {
                    try {
                        const message = JSON.parse(data.toString());
                        this.handleWebSocketMessage(message);
                    }
                    catch (error) {
                        console.error('[ContextEngine] Failed to parse WebSocket message:', error);
                    }
                });
                this.websocket.on('close', (code, reason) => {
                    console.log(`[ContextEngine] WebSocket connection closed: ${code} ${reason}`);
                    this.websocket = null;
                    // Attempt to reconnect
                    if (this.reconnectAttempts < this.maxReconnectAttempts) {
                        this.reconnectAttempts++;
                        console.log(`[ContextEngine] Attempting to reconnect (${this.reconnectAttempts}/${this.maxReconnectAttempts})...`);
                        setTimeout(() => {
                            this.connectWebSocket().catch(error => {
                                console.error('[ContextEngine] Reconnection failed:', error);
                            });
                        }, this.reconnectDelay * this.reconnectAttempts);
                    }
                });
                this.websocket.on('error', (error) => {
                    console.error('[ContextEngine] WebSocket error:', error);
                    reject(error);
                });
            }
            catch (error) {
                reject(error);
            }
        });
    }
    sendWebSocketMessage(message) {
        if (this.websocket && this.websocket.readyState === ws_1.default.OPEN) {
            this.websocket.send(JSON.stringify(message));
        }
    }
    handleWebSocketMessage(message) {
        console.log('[ContextEngine] WebSocket message received:', message.type);
        switch (message.type) {
            case 'context_updated':
                this.emit('contextUpdated', message.data);
                break;
            case 'suggestions_updated':
                this.emit('suggestionsUpdated', message.data);
                break;
            case 'analysis_complete':
                this.emit('analysisComplete', message.data);
                break;
            case 'error':
                this.emit('error', message.data);
                vscode.window.showErrorMessage(`Context Engine Error: ${message.data.message}`);
                break;
            default:
                console.log('[ContextEngine] Unknown message type:', message.type);
        }
    }
    on(event, handler) {
        if (!this.eventHandlers.has(event)) {
            this.eventHandlers.set(event, []);
        }
        this.eventHandlers.get(event).push(handler);
    }
    emit(event, data) {
        const handlers = this.eventHandlers.get(event);
        if (handlers) {
            handlers.forEach(handler => {
                try {
                    handler(data);
                }
                catch (error) {
                    console.error(`[ContextEngine] Error in event handler for ${event}:`, error);
                }
            });
        }
    }
    async analyzeFile(filePath) {
        try {
            // Send file change event via WebSocket for real-time processing
            this.sendWebSocketMessage({
                type: 'file_changed',
                data: {
                    file_path: filePath,
                    timestamp: new Date().toISOString()
                }
            });
            // Also make HTTP request for immediate analysis
            await this.httpClient.post('/analyze', {
                file_path: filePath,
                project_id: vscode.workspace.workspaceFolders?.[0]?.uri.fsPath || 'default'
            });
        }
        catch (error) {
            console.error('[ContextEngine] Failed to analyze file:', error);
            throw error;
        }
    }
    async getSuggestions(filePath) {
        try {
            const params = filePath ? { file_path: filePath } : {};
            const response = await this.httpClient.get('/suggestions', { params });
            return response.data.suggestions || [];
        }
        catch (error) {
            console.error('[ContextEngine] Failed to get suggestions:', error);
            throw error;
        }
    }
    async createContext(request) {
        try {
            await this.httpClient.post('/context', {
                title: request.title,
                context_type: request.contextType,
                content: request.content,
                description: request.description,
                project_id: vscode.workspace.workspaceFolders?.[0]?.uri.fsPath || 'default',
                metadata: {
                    file_path: request.filePath,
                    line_number: request.lineNumber,
                    source: 'vscode_extension'
                }
            });
        }
        catch (error) {
            console.error('[ContextEngine] Failed to create context:', error);
            throw error;
        }
    }
    async queryContext(query) {
        try {
            const response = await this.httpClient.post('/query', {
                query,
                project_id: vscode.workspace.workspaceFolders?.[0]?.uri.fsPath || 'default'
            });
            return response.data.results || [];
        }
        catch (error) {
            console.error('[ContextEngine] Failed to query context:', error);
            throw error;
        }
    }
    async getContextHealth() {
        try {
            const response = await this.httpClient.get('/health/context');
            return response.data;
        }
        catch (error) {
            console.error('[ContextEngine] Failed to get context health:', error);
            throw error;
        }
    }
    async getProjectInsights() {
        try {
            const projectId = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath || 'default';
            const response = await this.httpClient.get(`/insights/${encodeURIComponent(projectId)}`);
            return response.data;
        }
        catch (error) {
            console.error('[ContextEngine] Failed to get project insights:', error);
            throw error;
        }
    }
    async clearSuggestions(filePath) {
        try {
            const data = filePath ? { file_path: filePath } : {};
            await this.httpClient.post('/suggestions/clear', data);
        }
        catch (error) {
            console.error('[ContextEngine] Failed to clear suggestions:', error);
            throw error;
        }
    }
    disconnect() {
        if (this.websocket) {
            this.websocket.close();
            this.websocket = null;
        }
        console.log('[ContextEngine] Disconnected from server');
    }
    isConnected() {
        return this.websocket !== null && this.websocket.readyState === ws_1.default.OPEN;
    }
}
exports.ContextEngineClient = ContextEngineClient;
//# sourceMappingURL=contextEngineClient.js.map