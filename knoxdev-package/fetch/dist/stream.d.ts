export declare function toAsyncIterable(nodeReadable: NodeJS.ReadableStream): AsyncGenerator<Uint8Array>;
export declare function streamResponse(response: Response): AsyncGenerator<string>;
export declare function streamSse(response: Response): AsyncGenerator<any>;
export declare function streamJSON(response: Response): AsyncGenerator<any>;
