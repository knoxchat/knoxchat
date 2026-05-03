import { ConfigJson } from "../config-types/index.js";
import { ConfigYaml } from "./schemas/index.js";
export declare function convertJsonToYamlConfig(configJson: ConfigJson): ConfigYaml;
