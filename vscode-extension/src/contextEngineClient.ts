import * as vscode from 'vscode';
import axios, { AxiosInstance } from 'axios';
import WebSocket from 'ws';
import { ConfigurationManager } from './configurationManager';

export interface ContextSuggestion {
    id: string;
    title: string;
    description: string;
    suggestion_type: string;
    file_path?: string;
    line_number?: number;
    priority: 'Low' | 'Medium' | 'High' | 'Critical';
    actions: SuggestionAction[];
    metadata: Record<string, any>;
}

export interface SuggestionAction {
    action_type: string;
    title: string;
    description: string;
    parameters: Record<string, any>;
}

export interface CreateContextRequest {
    title: string;
    contextType: string;
    content: string;
    description: string;
    filePath: string;
    lineNumber: number;
}

export class ContextEngineClient {
    private httpClient: AxiosInstance;
    private websocket: WebSocket | null = null;
    private reconnectAttempts = 0;
    private maxReconnectAttempts = 5;
    private reconnectDelay = 1000;
    private eventHandlers: Map<string, Function[]> = new Map();

    constructor(private configManager: ConfigurationManager) {
        const serverUrl = this.configManager.getServerUrl();
        this.httpClient = axios.create({
            baseURL: serverUrl,
            timeout: 10000,
            headers: {
                'Content-Type': 'application/json',
                'User-Agent': 'VSCode-ContextEngine/1.0.0'
            }
        });

        // Setup request/response interceptors for logging
        this.httpClient.interceptors.request.use(
            (config) => {
                console.log(`[ContextEngine] HTTP Request: ${config.method?.toUpperCase()} ${config.url}`);
                return config;
            },
            (error) => {
                console.error('[ContextEngine] HTTP Request Error:', error);
                return Promise.reject(error);
            }
        );

        this.httpClient.interceptors.response.use(
            (response) => {
                console.log(`[ContextEngine] HTTP Response: ${response.status} ${response.config.url}`);
                return response;
            },
            (error) => {
                console.error('[ContextEngine] HTTP Response Error:', error.response?.status, error.message);
                return Promise.reject(error);
            }
        );
    }

    async connect(): Promise<void> {
        try {
            // Test HTTP connection first
            await this.httpClient.get('/health');
            console.log('[ContextEngine] HTTP connection established');

            // Establish WebSocket connection for real-time updates
            await this.connectWebSocket();
        } catch (error) {
            console.error('[ContextEngine] Failed to connect:', error);
            throw new Error(`Failed to connect to Context Engine server: ${error}`);
        }
    }

    private async connectWebSocket(): Promise<void> {
        const serverUrl = this.configManager.getServerUrl();
        const wsUrl = serverUrl.replace(/^http/, 'ws') + '/ws';

        return new Promise((resolve, reject) => {
            try {
                this.websocket = new WebSocket(wsUrl);

                this.websocket!.on('open', () => {
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

                this.websocket!.on('message', (data: WebSocket.Data) => {
                    try {
                        const message = JSON.parse(data.toString());
                        this.handleWebSocketMessage(message);
                    } catch (error) {
                        console.error('[ContextEngine] Failed to parse WebSocket message:', error);
                    }
                });

                this.websocket!.on('close', (code: number, reason: string) => {
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

                this.websocket!.on('error', (error: Error) => {
                    console.error('[ContextEngine] WebSocket error:', error);
                    reject(error);
                });

            } catch (error) {
                reject(error);
            }
        });
    }

    private sendWebSocketMessage(message: any): void {
        if (this.websocket && this.websocket.readyState === WebSocket.OPEN) {
            this.websocket.send(JSON.stringify(message));
        }
    }

    private handleWebSocketMessage(message: any): void {
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

    on(event: string, handler: Function): void {
        if (!this.eventHandlers.has(event)) {
            this.eventHandlers.set(event, []);
        }
        this.eventHandlers.get(event)!.push(handler);
    }

    private emit(event: string, data: any): void {
        const handlers = this.eventHandlers.get(event);
        if (handlers) {
            handlers.forEach(handler => {
                try {
                    handler(data);
                } catch (error) {
                    console.error(`[ContextEngine] Error in event handler for ${event}:`, error);
                }
            });
        }
    }

    async analyzeFile(filePath: string): Promise<void> {
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
        } catch (error) {
            console.error('[ContextEngine] Failed to analyze file:', error);
            throw error;
        }
    }

    async getSuggestions(filePath?: string): Promise<ContextSuggestion[]> {
        try {
            const params = filePath ? { file_path: filePath } : {};
            const response = await this.httpClient.get('/suggestions', { params });
            return response.data.suggestions || [];
        } catch (error) {
            console.error('[ContextEngine] Failed to get suggestions:', error);
            throw error;
        }
    }

    async createContext(request: CreateContextRequest): Promise<void> {
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
        } catch (error) {
            console.error('[ContextEngine] Failed to create context:', error);
            throw error;
        }
    }

    async queryContext(query: string): Promise<any[]> {
        try {
            const response = await this.httpClient.post('/query', {
                query,
                project_id: vscode.workspace.workspaceFolders?.[0]?.uri.fsPath || 'default'
            });
            return response.data.results || [];
        } catch (error) {
            console.error('[ContextEngine] Failed to query context:', error);
            throw error;
        }
    }

    async getContextHealth(): Promise<any> {
        try {
            const response = await this.httpClient.get('/health/context');
            return response.data;
        } catch (error) {
            console.error('[ContextEngine] Failed to get context health:', error);
            throw error;
        }
    }

    async getProjectInsights(): Promise<any> {
        try {
            const projectId = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath || 'default';
            const response = await this.httpClient.get(`/insights/${encodeURIComponent(projectId)}`);
            return response.data;
        } catch (error) {
            console.error('[ContextEngine] Failed to get project insights:', error);
            throw error;
        }
    }

    async clearSuggestions(filePath?: string): Promise<void> {
        try {
            const data = filePath ? { file_path: filePath } : {};
            await this.httpClient.post('/suggestions/clear', data);
        } catch (error) {
            console.error('[ContextEngine] Failed to clear suggestions:', error);
            throw error;
        }
    }

    disconnect(): void {
        if (this.websocket) {
            this.websocket.close();
            this.websocket = null;
        }
        console.log('[ContextEngine] Disconnected from server');
    }

    isConnected(): boolean {
        return this.websocket !== null && this.websocket.readyState === WebSocket.OPEN;
    }
}