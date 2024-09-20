import Anthropic from '@anthropic-ai/sdk';
import { anthropicClient } from './api.js';

export type Tool<T> = (userPrompt: string) => Promise<T>;

export const makeTool = <T>(systemPrompt: string, definition: Anthropic.Messages.Tool): Tool<T> => {
    return async (userPrompt) => {
        const message = await anthropicClient.messages.create({
            max_tokens: 1024,
            system: systemPrompt,
            messages: [
                { role: 'user', content: userPrompt }
            ],
            model: 'claude-3-5-sonnet-20240620',
            tool_choice: { "type": "tool", "name": definition.name },
            tools: [
                definition
            ]
        });

        return (message.content[0] as any).input as T;
    };
}
