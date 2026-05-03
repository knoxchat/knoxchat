import { ConfigJson } from "../config-types/index.js";
import { ConfigYaml } from "./schemas/index.js";
import { ModelRole } from "./schemas/models.js";

type ModelYaml = NonNullable<ConfigYaml["models"]>[number];
type ContextYaml = NonNullable<ConfigYaml["context"]>[number];
type PromptYaml = NonNullable<ConfigYaml["prompts"]>[number];

function convertModel(
  m: ConfigJson["models"][number],
  roles: ModelRole[],
): ModelYaml {
  return {
    name: m.title,
    provider: m.provider,
    model: m.model,
    apiKey: m.apiKey,
    apiBase: m.apiBase,
    roles,
    requestOptions: m.requestOptions,
    defaultCompletionOptions: m.completionOptions,
  };
}

function convertContext(configJson: ConfigJson): ContextYaml[] {
  const context: ContextYaml[] =
    configJson.contextProviders?.map((ctx) => {
      return {
        uses: `builtin/${ctx.name}`,
        with: ctx.params,
      };
    }) ?? [];

  return context;
}

function convertCustomCommand(
  cmd: NonNullable<ConfigJson["customCommands"]>[number],
): PromptYaml {
  return {
    name: cmd.name,
    description: cmd.description,
    prompt: (cmd as any).prompt,
  };
}

export function convertJsonToYamlConfig(configJson: ConfigJson): ConfigYaml {
  // models
  const models = configJson.models.map((m) => convertModel(m, ["chat"]));
  const autocompleteModels = Array.isArray(configJson.tabAutocompleteModel)
    ? configJson.tabAutocompleteModel
    : configJson.tabAutocompleteModel
      ? [configJson.tabAutocompleteModel]
      : [];
  models.push(
    ...autocompleteModels.map((m) => convertModel(m, ["autocomplete"])),
  );



  // context
  const context = convertContext(configJson);

  // prompts
  const prompts = configJson.customCommands?.map(convertCustomCommand);


  const configYaml: ConfigYaml = {
    name: "Knox Config",
    version: "0.0.1",
    models,
    context,
    rules: configJson.systemMessage ? [configJson.systemMessage] : undefined,
    prompts,
  };

  return configYaml;
}
