import { SecretType } from "./SecretResult.js";
import { packageSlugsEqual } from "./slugs.js";
export function getLocationsToLook(assistantSlug, blockSlug, currentUserSlug, secretName, orgScopeSlug) {
    const locationsToLook = [
        ...(blockSlug
            ? [
                // Models Add-On
                {
                    secretType: SecretType.ModelsAddOn,
                    secretName,
                    blockSlug,
                },
                // Block
                {
                    secretType: SecretType.Package,
                    packageSlug: blockSlug,
                    secretName,
                },
            ]
            : []),
        // Assistant
        {
            secretType: SecretType.Package,
            packageSlug: assistantSlug,
            secretName,
        },
        // Organization that owns assistant (org secrets can only be used in assistants owned by that org)
        ...(orgScopeSlug
            ? [
                {
                    secretType: SecretType.Organization,
                    orgSlug: orgScopeSlug,
                    secretName,
                },
            ]
            : []),
        // User
        {
            secretType: SecretType.User,
            userSlug: currentUserSlug,
            secretName,
        },
    ];
    return locationsToLook;
}
export function listAvailableSecrets(userSecretNames, orgSecretNames, assistantSecretNames, blockSecretNames, assistantSlug, blockSlug, currentUserSlug, orgScopeSlug) {
    // Create a set of all secret names
    const allSecretNames = new Set([
        ...userSecretNames,
        ...orgSecretNames,
        ...assistantSecretNames,
        ...blockSecretNames,
    ]);
    // Use the resolution order to get a single SecretLocation for each secret name
    const secretLocations = [];
    for (const secretName of allSecretNames) {
        // Get the order of places to look
        const locationsToLook = getLocationsToLook(assistantSlug, blockSlug, currentUserSlug, secretName, orgScopeSlug);
        // Go through the locations one by one
        for (const secretLocation of locationsToLook) {
            // "Looking in a location" in this case means looking through one of the lists of secret names
            // First we get that list of secret names
            let secretNamesList = [];
            switch (secretLocation.secretType) {
                case SecretType.User:
                    secretNamesList = userSecretNames;
                    break;
                case SecretType.Organization:
                    secretNamesList = orgSecretNames;
                    break;
                case SecretType.Package:
                    if (packageSlugsEqual(secretLocation.packageSlug, assistantSlug)) {
                        secretNamesList = assistantSecretNames;
                    }
                    else if (blockSlug &&
                        packageSlugsEqual(secretLocation.packageSlug, blockSlug)) {
                        secretNamesList = blockSecretNames;
                    }
                    break;
            }
            // Then we look through that list for the matching secret name
            if (secretNamesList) {
                const matchingSecretName = secretNamesList.find((secretName) => secretName === secretLocation.secretName);
                if (matchingSecretName) {
                    // If we find a matching secret name, we add the location to the list
                    secretLocations.push(secretLocation);
                    break;
                }
            }
        }
    }
    return secretLocations;
}
export async function resolveFQSN(currentUserSlug, fqsn, platformSecretStore, orgScopeSlug) {
    // First create the list of secret locations to try in order
    const assistantSlug = fqsn.packageSlugs[0];
    const blockSlug = fqsn.packageSlugs[1];
    const locationsToLook = getLocationsToLook(assistantSlug, blockSlug, currentUserSlug, fqsn.secretName, orgScopeSlug);
    // Then try to get the secret from each location
    for (const secretLocation of locationsToLook) {
        const secret = await platformSecretStore.getSecretFromSecretLocation(secretLocation);
        if (secret) {
            if (secretLocation.secretType === SecretType.User) {
                // Only user secret values get sent back to client
                return {
                    found: true,
                    fqsn,
                    secretLocation,
                    value: secret,
                };
            }
            else if (secretLocation.secretType !== SecretType.NotFound) {
                return {
                    found: true,
                    fqsn,
                    secretLocation,
                };
            }
        }
    }
    return {
        found: false,
        secretLocation: {
            secretName: fqsn.secretName,
            secretType: SecretType.NotFound,
        },
        fqsn,
    };
}
