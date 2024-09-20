import Anthropic from '@anthropic-ai/sdk';

export const anthropicClient = new Anthropic({
    dangerouslyAllowBrowser: true,
    apiKey: "ANTHROPIC_API_KEY_GOES_HERE"
});
