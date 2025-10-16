export interface PackageSlug {
    ownerSlug: string;
    packageSlug: string;
}
export interface FullSlug extends PackageSlug {
    versionSlug: string;
}
export declare enum VirtualTags {
    Latest = "latest"
}
export declare function encodePackageSlug(packageSlug: PackageSlug): string;
export declare function decodePackageSlug(pkgSlug: string): PackageSlug;
export declare function encodeFullSlug(fullSlug: FullSlug): string;
export declare function packageSlugsEqual(pkgSlug1: PackageSlug, pkgSlug2: PackageSlug): boolean;
export declare function decodeFullSlug(fullSlug: string): FullSlug;
/**
 * FQSN = Fully Qualified Secret Name
 */
export interface FQSN {
    packageSlugs: PackageSlug[];
    secretName: string;
}
export declare function encodeFQSN(fqsn: FQSN): string;
export declare function decodeFQSN(fqsn: string): FQSN;
