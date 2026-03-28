import { Registry } from "./interfaces/index.js";
import { FullSlug } from "./interfaces/slugs.js";
export declare class RegistryClient implements Registry {
    private readonly accessToken?;
    private readonly apiBase;
    constructor(accessToken?: string | undefined, apiBase?: string);
    getContent(fullSlug: FullSlug): Promise<string>;
}
