export var VirtualTags;
(function (VirtualTags) {
    VirtualTags["Latest"] = "latest";
})(VirtualTags || (VirtualTags = {}));
export function encodePackageSlug(packageSlug) {
    return `${packageSlug.ownerSlug}/${packageSlug.packageSlug}`;
}
export function decodePackageSlug(pkgSlug) {
    const [ownerSlug, packageSlug] = pkgSlug.split("/");
    return {
        ownerSlug,
        packageSlug,
    };
}
export function encodeFullSlug(fullSlug) {
    return `${fullSlug.ownerSlug}/${fullSlug.packageSlug}@${fullSlug.versionSlug}`;
}
export function packageSlugsEqual(pkgSlug1, pkgSlug2) {
    return (pkgSlug1.ownerSlug === pkgSlug2.ownerSlug &&
        pkgSlug1.packageSlug === pkgSlug2.packageSlug);
}
export function decodeFullSlug(fullSlug) {
    const [ownerSlug, packageSlug, versionSlug] = fullSlug.split(/[/@]/);
    return {
        ownerSlug,
        packageSlug,
        versionSlug: versionSlug || VirtualTags.Latest,
    };
}
export function encodeFQSN(fqsn) {
    const parts = [...fqsn.packageSlugs.map(encodePackageSlug), fqsn.secretName];
    return parts.join("/");
}
export function decodeFQSN(fqsn) {
    const parts = fqsn.split("/");
    const secretName = parts.pop();
    const packageSlugs = [];
    // Process parts two at a time to decode package slugs
    for (let i = 0; i < parts.length; i += 2) {
        if (i + 1 >= parts.length) {
            throw new Error("Invalid FQSN format: package slug must have two parts");
        }
        packageSlugs.push({
            ownerSlug: parts[i],
            packageSlug: parts[i + 1],
        });
    }
    return { packageSlugs, secretName };
}
