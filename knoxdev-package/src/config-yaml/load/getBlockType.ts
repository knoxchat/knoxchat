import { ConfigYaml } from "../schemas/index.js";

export type BlockType =
  | "models"
  | "context"
  | "data"
  | "rules"
  | "prompts";

export function getBlockType(block: ConfigYaml): BlockType | undefined {
  if (block.context?.length) {
    return "context";
  } else if (block.models?.length) {
    return "models";
  } else if (block.data?.length) {
    return "data";
  } else if (block.rules?.length) {
    return "rules";
  } else if (block.prompts?.length) {
    return "prompts";
  } else {
    return undefined;
  }
}
