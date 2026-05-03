import * as z from "zod";
import { blockItemWrapperSchema } from "../schemas/index.js";
export declare const isBlockItemWrapper: (block: unknown) => block is z.infer<ReturnType<typeof blockItemWrapperSchema<z.ZodObject<any>>>>;
