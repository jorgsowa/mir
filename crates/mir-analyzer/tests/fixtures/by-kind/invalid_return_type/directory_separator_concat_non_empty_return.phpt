===file===
<?php
/**
 * M19 (B8 lineage): `DIRECTORY_SEPARATOR` is always "/" or "\", so it must
 * type as `non-empty-string`. Two concat shapes depend on that:
 *
 * 1. `realpath() . DIRECTORY_SEPARATOR . basename()` — PHPUnit's
 *    Util\Filesystem::resolveStreamOrFile() declares `false|non-empty-string`;
 *    before the fix the separator typed `string` (environment-dependent widen)
 *    and the concat degraded to `string`, failing the return-type check
 *    (InvalidReturnType).
 * 2. Plain `string . DIRECTORY_SEPARATOR . string` — the provably non-empty
 *    separator refines the whole concat to `non-empty-string`.
 */

/**
 * @param non-empty-string $path
 *
 * @return false|non-empty-string
 */
function resolve_stream_or_file(string $path): false|string {
    if (str_starts_with($path, 'php://') || str_starts_with($path, 'socket://')) {
        return $path;
    }

    $directory = dirname($path);

    if (is_dir($directory)) {
        $joined = realpath($directory) . DIRECTORY_SEPARATOR . basename($path);
        /** @mir-check $joined is non-empty-string */
        return $joined;
    }

    return false;
}

function join_with_separator(string $left, string $right): string {
    $joined = $left . DIRECTORY_SEPARATOR . $right;
    /** @mir-check $joined is non-empty-string */
    return $joined;
}
===expect===
