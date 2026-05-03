import { chatChunk, chatCompletion } from "../util.js";
const MOCK_RESPONSE = "This is a mock response from the OpenAI API. It can be returned all at once or streamed chunk by chunk.";
export class MockApi {
    async chatCompletionNonStream(body, signal) {
        await new Promise((resolve) => setTimeout(resolve, 800));
        const lastMessage = body.messages[body.messages.length - 1].content;
        const content = !lastMessage
            ? MOCK_RESPONSE
            : typeof lastMessage === "string"
                ? lastMessage
                : lastMessage[0].type === "text"
                    ? lastMessage[0].text
                    : MOCK_RESPONSE;
        return chatCompletion({
            content,
            model: body.model,
        });
    }
    async *chatCompletionStream(body, signal) {
        const lastMessage = body.messages[body.messages.length - 1].content;
        const content = !lastMessage
            ? MOCK_RESPONSE
            : typeof lastMessage === "string"
                ? lastMessage
                : lastMessage[0].type === "text"
                    ? lastMessage[0].text
                    : MOCK_RESPONSE;
        const chunks = content.split(" ");
        for (const chunk of chunks) {
            await new Promise((resolve) => setTimeout(resolve, 100));
            yield chatChunk({
                content: chunk + " ",
                model: body.model,
            });
        }
    }
    async completionNonStream(body, signal) {
        await new Promise((resolve) => setTimeout(resolve, 800));
        return {
            id: "mock-completion",
            object: "text_completion",
            created: Date.now(),
            model: body.model,
            choices: [
                {
                    text: body.prompt,
                    index: 0,
                    finish_reason: "stop",
                },
            ],
        };
    }
    async *completionStream(body, signal) {
        const chunks = body.prompt.split(" ");
        for (const chunk of chunks) {
            await new Promise((resolve) => setTimeout(resolve, 100));
            yield {
                id: "mock-chunk",
                object: "text_completion",
                created: Date.now(),
                model: body.model,
                choices: [
                    {
                        text: chunk + " ",
                        index: 0,
                        finish_reason: "stop",
                    },
                ],
            };
        }
    }
    async *fimStream(body, signal) {
        const chunks = body.prompt.split(" ");
        for (const chunk of chunks) {
            await new Promise((resolve) => setTimeout(resolve, 100));
            yield chatChunk({
                content: chunk + " ",
                model: body.model,
            });
        }
    }
    async list() {
        return [
            {
                id: "mock-model",
                created: Date.now(),
                object: "model",
                owned_by: "mock",
            },
        ];
    }
}
