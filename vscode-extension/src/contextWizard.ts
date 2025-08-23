import * as vscode from 'vscode';
import { ContextEngineClient } from './contextEngineClient';

export interface ContextTemplate {
    id: string;
    name: string;
    description: string;
    type: string;
    fields: TemplateField[];
    tags: string[];
    icon: string;
}

export interface TemplateField {
    name: string;
    label: string;
    type: 'text' | 'textarea' | 'select' | 'multiselect' | 'boolean' | 'number';
    required: boolean;
    placeholder?: string;
    options?: string[];
    defaultValue?: any;
    validation?: string;
}

export interface WizardStep {
    title: string;
    description: string;
    fields: TemplateField[];
    validate?: (data: any) => string | null;
}

export class ContextWizard {
    private static readonly TEMPLATES: ContextTemplate[] = [
        {
            id: 'business_rule',
            name: 'Business Rule',
            description: 'Document business logic and rules',
            type: 'business_rule',
            icon: 'law',
            tags: ['business', 'rules', 'logic'],
            fields: [
                { name: 'title', label: 'Rule Title', type: 'text', required: true, placeholder: 'e.g., User Authentication Rule' },
                { name: 'description', label: 'Description', type: 'textarea', required: true, placeholder: 'Detailed description of the business rule...' },
                { name: 'conditions', label: 'Conditions', type: 'textarea', required: false, placeholder: 'When does this rule apply?' },
                { name: 'actions', label: 'Actions', type: 'textarea', required: false, placeholder: 'What actions should be taken?' },
                { name: 'exceptions', label: 'Exceptions', type: 'textarea', required: false, placeholder: 'Any exceptions to this rule?' },
                { name: 'priority', label: 'Priority', type: 'select', required: true, options: ['Low', 'Medium', 'High', 'Critical'], defaultValue: 'Medium' },
                { name: 'stakeholders', label: 'Stakeholders', type: 'text', required: false, placeholder: 'Who needs to know about this rule?' }
            ]
        },
        {
            id: 'architectural_decision',
            name: 'Architectural Decision',
            description: 'Record architectural decisions and rationale',
            type: 'architectural_decision',
            icon: 'organization',
            tags: ['architecture', 'decision', 'design'],
            fields: [
                { name: 'title', label: 'Decision Title', type: 'text', required: true, placeholder: 'e.g., Use REST API for Service Communication' },
                { name: 'status', label: 'Status', type: 'select', required: true, options: ['Proposed', 'Accepted', 'Deprecated', 'Superseded'], defaultValue: 'Proposed' },
                { name: 'context', label: 'Context', type: 'textarea', required: true, placeholder: 'What is the context of this decision?' },
                { name: 'decision', label: 'Decision', type: 'textarea', required: true, placeholder: 'What is the change that we are proposing?' },
                { name: 'rationale', label: 'Rationale', type: 'textarea', required: true, placeholder: 'Why are we making this decision?' },
                { name: 'consequences', label: 'Consequences', type: 'textarea', required: false, placeholder: 'What are the positive and negative consequences?' },
                { name: 'alternatives', label: 'Alternatives', type: 'textarea', required: false, placeholder: 'What other options were considered?' },
                { name: 'related_decisions', label: 'Related Decisions', type: 'text', required: false, placeholder: 'Links to related decisions' }
            ]
        },
        {
            id: 'api_specification',
            name: 'API Specification',
            description: 'Document API endpoints and contracts',
            type: 'api_specification',
            icon: 'globe',
            tags: ['api', 'specification', 'contract'],
            fields: [
                { name: 'title', label: 'API Title', type: 'text', required: true, placeholder: 'e.g., User Management API' },
                { name: 'version', label: 'Version', type: 'text', required: true, placeholder: 'e.g., v1.0.0', defaultValue: 'v1.0.0' },
                { name: 'base_url', label: 'Base URL', type: 'text', required: true, placeholder: 'e.g., https://api.example.com/v1' },
                { name: 'description', label: 'Description', type: 'textarea', required: true, placeholder: 'What does this API do?' },
                { name: 'authentication', label: 'Authentication', type: 'select', required: true, options: ['None', 'API Key', 'Bearer Token', 'OAuth2', 'Basic Auth'], defaultValue: 'Bearer Token' },
                { name: 'endpoints', label: 'Key Endpoints', type: 'textarea', required: false, placeholder: 'List main endpoints (one per line)' },
                { name: 'rate_limits', label: 'Rate Limits', type: 'text', required: false, placeholder: 'e.g., 1000 requests per hour' },
                { name: 'error_handling', label: 'Error Handling', type: 'textarea', required: false, placeholder: 'How are errors handled?' }
            ]
        },
        {
            id: 'performance_requirement',
            name: 'Performance Requirement',
            description: 'Define performance criteria and benchmarks',
            type: 'performance_requirement',
            icon: 'dashboard',
            tags: ['performance', 'requirement', 'benchmark'],
            fields: [
                { name: 'title', label: 'Requirement Title', type: 'text', required: true, placeholder: 'e.g., API Response Time Requirement' },
                { name: 'description', label: 'Description', type: 'textarea', required: true, placeholder: 'Detailed description of the performance requirement...' },
                { name: 'metric', label: 'Metric', type: 'select', required: true, options: ['Response Time', 'Throughput', 'Memory Usage', 'CPU Usage', 'Disk I/O', 'Network I/O', 'Concurrent Users'], defaultValue: 'Response Time' },
                { name: 'target_value', label: 'Target Value', type: 'text', required: true, placeholder: 'e.g., < 200ms, > 1000 req/s' },
                { name: 'measurement_method', label: 'Measurement Method', type: 'textarea', required: false, placeholder: 'How will this be measured?' },
                { name: 'test_conditions', label: 'Test Conditions', type: 'textarea', required: false, placeholder: 'Under what conditions should this be tested?' },
                { name: 'priority', label: 'Priority', type: 'select', required: true, options: ['Low', 'Medium', 'High', 'Critical'], defaultValue: 'Medium' }
            ]
        },
        {
            id: 'security_policy',
            name: 'Security Policy',
            description: 'Document security policies and requirements',
            type: 'security_policy',
            icon: 'shield',
            tags: ['security', 'policy', 'compliance'],
            fields: [
                { name: 'title', label: 'Policy Title', type: 'text', required: true, placeholder: 'e.g., Data Encryption Policy' },
                { name: 'description', label: 'Description', type: 'textarea', required: true, placeholder: 'Detailed description of the security policy...' },
                { name: 'scope', label: 'Scope', type: 'textarea', required: true, placeholder: 'What does this policy apply to?' },
                { name: 'requirements', label: 'Requirements', type: 'textarea', required: true, placeholder: 'What are the specific requirements?' },
                { name: 'compliance_standards', label: 'Compliance Standards', type: 'multiselect', required: false, options: ['GDPR', 'HIPAA', 'SOX', 'PCI DSS', 'ISO 27001', 'NIST'], placeholder: 'Select applicable standards' },
                { name: 'implementation_notes', label: 'Implementation Notes', type: 'textarea', required: false, placeholder: 'How should this be implemented?' },
                { name: 'review_frequency', label: 'Review Frequency', type: 'select', required: true, options: ['Monthly', 'Quarterly', 'Semi-annually', 'Annually'], defaultValue: 'Quarterly' }
            ]
        }
    ];

    constructor(private contextClient: ContextEngineClient) {}

    async showWizard(): Promise<void> {
        // Step 1: Select template
        const template = await this.selectTemplate();
        if (!template) return;

        // Step 2: Collect data using multi-step wizard
        const contextData = await this.collectContextData(template);
        if (!contextData) return;

        // Step 3: Review and confirm
        const confirmed = await this.reviewAndConfirm(template, contextData);
        if (!confirmed) return;

        // Step 4: Create context
        await this.createContext(template, contextData);
    }

    private async selectTemplate(): Promise<ContextTemplate | undefined> {
        const items = ContextWizard.TEMPLATES.map(template => ({
            label: `$(${template.icon}) ${template.name}`,
            description: template.description,
            detail: `Tags: ${template.tags.join(', ')}`,
            template
        }));

        const selected = await vscode.window.showQuickPick(items, {
            placeHolder: 'Select a context template',
            matchOnDescription: true,
            matchOnDetail: true
        });

        return selected?.template;
    }

    private async collectContextData(template: ContextTemplate): Promise<any | undefined> {
        const data: any = {};
        const steps = this.createWizardSteps(template);

        for (let i = 0; i < steps.length; i++) {
            const step = steps[i];
            const stepData = await this.executeWizardStep(step, i + 1, steps.length);
            
            if (!stepData) {
                // User cancelled
                return undefined;
            }

            Object.assign(data, stepData);
        }

        return data;
    }

    private createWizardSteps(template: ContextTemplate): WizardStep[] {
        // Group fields into logical steps
        const basicFields = template.fields.filter(f => ['title', 'description', 'type'].includes(f.name));
        const detailFields = template.fields.filter(f => !['title', 'description', 'type'].includes(f.name) && f.required);
        const optionalFields = template.fields.filter(f => !['title', 'description', 'type'].includes(f.name) && !f.required);

        const steps: WizardStep[] = [];

        if (basicFields.length > 0) {
            steps.push({
                title: 'Basic Information',
                description: 'Provide basic information about the context',
                fields: basicFields,
                validate: (data) => {
                    const requiredFields = basicFields.filter(f => f.required);
                    for (const field of requiredFields) {
                        if (!data[field.name] || data[field.name].trim() === '') {
                            return `${field.label} is required`;
                        }
                    }
                    return null;
                }
            });
        }

        if (detailFields.length > 0) {
            steps.push({
                title: 'Required Details',
                description: 'Provide required details for the context',
                fields: detailFields,
                validate: (data) => {
                    for (const field of detailFields) {
                        if (!data[field.name] || data[field.name].trim() === '') {
                            return `${field.label} is required`;
                        }
                    }
                    return null;
                }
            });
        }

        if (optionalFields.length > 0) {
            steps.push({
                title: 'Additional Information',
                description: 'Provide additional optional information',
                fields: optionalFields
            });
        }

        return steps;
    }

    private async executeWizardStep(step: WizardStep, stepNumber: number, totalSteps: number): Promise<any | undefined> {
        const data: any = {};

        // Show step progress
        const progress = `Step ${stepNumber} of ${totalSteps}`;
        
        for (const field of step.fields) {
            const value = await this.collectFieldValue(field, progress);
            if (value === undefined) {
                // User cancelled
                return undefined;
            }
            data[field.name] = value;
        }

        // Validate step data
        if (step.validate) {
            const error = step.validate(data);
            if (error) {
                vscode.window.showErrorMessage(error);
                // Retry the step
                return this.executeWizardStep(step, stepNumber, totalSteps);
            }
        }

        return data;
    }

    private async collectFieldValue(field: TemplateField, progress: string): Promise<any> {
        const prompt = `${progress} - ${field.label}`;
        
        switch (field.type) {
            case 'text':
                return await vscode.window.showInputBox({
                    prompt,
                    placeHolder: field.placeholder,
                    value: field.defaultValue,
                    validateInput: field.validation ? this.createValidator(field.validation) : undefined
                });

            case 'textarea':
                // For textarea, we'll use a multi-line input box
                const textValue = await vscode.window.showInputBox({
                    prompt: `${prompt} (Use Shift+Enter for new lines)`,
                    placeHolder: field.placeholder,
                    value: field.defaultValue
                });
                return textValue;

            case 'select':
                if (!field.options) return field.defaultValue;
                const selected = await vscode.window.showQuickPick(field.options, {
                    placeHolder: prompt
                });
                return selected || field.defaultValue;

            case 'multiselect':
                if (!field.options) return [];
                const multiSelected = await vscode.window.showQuickPick(
                    field.options.map(option => ({ label: option, picked: false })),
                    {
                        placeHolder: prompt,
                        canPickMany: true
                    }
                );
                return multiSelected?.map(item => item.label) || [];

            case 'boolean':
                const boolSelected = await vscode.window.showQuickPick(['Yes', 'No'], {
                    placeHolder: prompt
                });
                return boolSelected === 'Yes';

            case 'number':
                const numberValue = await vscode.window.showInputBox({
                    prompt,
                    placeHolder: field.placeholder,
                    value: field.defaultValue?.toString(),
                    validateInput: (value) => {
                        if (value && isNaN(Number(value))) {
                            return 'Please enter a valid number';
                        }
                        return null;
                    }
                });
                return numberValue ? Number(numberValue) : field.defaultValue;

            default:
                return field.defaultValue;
        }
    }

    private createValidator(validation: string): (value: string) => string | null {
        return (value: string) => {
            try {
                const regex = new RegExp(validation);
                if (!regex.test(value)) {
                    return 'Invalid format';
                }
            } catch (error) {
                // Invalid regex, skip validation
            }
            return null;
        };
    }

    private async reviewAndConfirm(template: ContextTemplate, data: any): Promise<boolean> {
        const panel = vscode.window.createWebviewPanel(
            'contextReview',
            'Review Context',
            vscode.ViewColumn.Beside,
            {
                enableScripts: true,
                retainContextWhenHidden: true
            }
        );

        panel.webview.html = this.getReviewWebviewContent(template, data);

        return new Promise((resolve) => {
            panel.webview.onDidReceiveMessage(
                message => {
                    switch (message.command) {
                        case 'confirm':
                            panel.dispose();
                            resolve(true);
                            return;
                        case 'cancel':
                            panel.dispose();
                            resolve(false);
                            return;
                    }
                }
            );

            panel.onDidDispose(() => {
                resolve(false);
            });
        });
    }

    private async createContext(template: ContextTemplate, data: any): Promise<void> {
        try {
            const activeEditor = vscode.window.activeTextEditor;
            
            await this.contextClient.createContext({
                title: data.title,
                contextType: template.type,
                content: this.formatContextContent(template, data),
                description: data.description,
                filePath: activeEditor?.document.uri.fsPath || '',
                lineNumber: activeEditor?.selection.start.line || 0
            });

            vscode.window.showInformationMessage(`${template.name} created successfully!`);
        } catch (error) {
            vscode.window.showErrorMessage(`Failed to create context: ${error}`);
        }
    }

    private formatContextContent(template: ContextTemplate, data: any): string {
        const lines: string[] = [];
        
        lines.push(`# ${data.title}`);
        lines.push('');
        lines.push(`**Type:** ${template.name}`);
        lines.push('');

        for (const field of template.fields) {
            if (data[field.name] && field.name !== 'title') {
                lines.push(`**${field.label}:**`);
                if (Array.isArray(data[field.name])) {
                    lines.push(data[field.name].map((item: string) => `- ${item}`).join('\n'));
                } else {
                    lines.push(data[field.name].toString());
                }
                lines.push('');
            }
        }

        return lines.join('\n');
    }

    private getReviewWebviewContent(template: ContextTemplate, data: any): string {
        const fields = template.fields
            .filter(field => data[field.name])
            .map(field => {
                let value = data[field.name];
                if (Array.isArray(value)) {
                    value = value.join(', ');
                }
                return `
                    <div class="field">
                        <label>${field.label}:</label>
                        <div class="value">${value}</div>
                    </div>
                `;
            })
            .join('');

        return `
            <!DOCTYPE html>
            <html>
            <head>
                <meta charset="UTF-8">
                <meta name="viewport" content="width=device-width, initial-scale=1.0">
                <title>Review Context</title>
                <style>
                    body {
                        font-family: var(--vscode-font-family);
                        padding: 20px;
                        color: var(--vscode-foreground);
                        background-color: var(--vscode-editor-background);
                    }
                    .header {
                        border-bottom: 1px solid var(--vscode-panel-border);
                        padding-bottom: 15px;
                        margin-bottom: 20px;
                    }
                    .template-info {
                        display: flex;
                        align-items: center;
                        margin-bottom: 10px;
                    }
                    .template-icon {
                        margin-right: 10px;
                        font-size: 24px;
                    }
                    .field {
                        margin-bottom: 15px;
                        padding: 10px;
                        background-color: var(--vscode-editor-inactiveSelectionBackground);
                        border-radius: 4px;
                    }
                    .field label {
                        font-weight: bold;
                        display: block;
                        margin-bottom: 5px;
                        color: var(--vscode-textLink-foreground);
                    }
                    .field .value {
                        white-space: pre-wrap;
                        word-wrap: break-word;
                    }
                    .actions {
                        margin-top: 30px;
                        text-align: center;
                    }
                    .button {
                        background-color: var(--vscode-button-background);
                        color: var(--vscode-button-foreground);
                        border: none;
                        padding: 10px 20px;
                        margin: 0 10px;
                        border-radius: 4px;
                        cursor: pointer;
                        font-size: 14px;
                    }
                    .button:hover {
                        background-color: var(--vscode-button-hoverBackground);
                    }
                    .button.secondary {
                        background-color: var(--vscode-button-secondaryBackground);
                        color: var(--vscode-button-secondaryForeground);
                    }
                    .button.secondary:hover {
                        background-color: var(--vscode-button-secondaryHoverBackground);
                    }
                </style>
            </head>
            <body>
                <div class="header">
                    <div class="template-info">
                        <span class="template-icon">📋</span>
                        <div>
                            <h1>Review ${template.name}</h1>
                            <p>${template.description}</p>
                        </div>
                    </div>
                </div>
                
                <div class="content">
                    ${fields}
                </div>
                
                <div class="actions">
                    <button class="button" onclick="confirm()">Create Context</button>
                    <button class="button secondary" onclick="cancel()">Cancel</button>
                </div>

                <script>
                    const vscode = acquireVsCodeApi();
                    
                    function confirm() {
                        vscode.postMessage({ command: 'confirm' });
                    }
                    
                    function cancel() {
                        vscode.postMessage({ command: 'cancel' });
                    }
                </script>
            </body>
            </html>
        `;
    }
}