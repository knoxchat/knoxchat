import { ConfigYaml } from "../schemas/index.js";
export type BlockType = "models" | "context" | "mcpServers" | "data" | "rules" | "prompts";
export declare function getBlockType(block: ConfigYaml): BlockType | undefined;
