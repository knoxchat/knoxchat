import { ConfigYaml } from "./schemas/index.js";
export interface ConfigValidationError {
    fatal: boolean;
    message: string;
}
export interface ConfigResult<T> {
    config: T | undefined;
    errors: ConfigValidationError[] | undefined;
    configLoadInterrupted: boolean;
}
export declare function validateConfigYaml(config: ConfigYaml): ConfigValidationError[];
