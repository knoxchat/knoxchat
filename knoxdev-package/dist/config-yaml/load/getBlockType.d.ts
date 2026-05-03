import { ConfigYaml } from "../schemas/index.js";
export type BlockType = "models" | "context" | "data" | "rules" | "prompts";
export declare function getBlockType(block: ConfigYaml): BlockType | undefined;
