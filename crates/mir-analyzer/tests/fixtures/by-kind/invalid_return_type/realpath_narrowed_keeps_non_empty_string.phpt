===description===
M18: realpath()'s success case was typed plain `string` (a resolved path
can never be empty), so narrowing its `string|false` result with
`!== false` only recovered `string`, flagging InvalidReturnType against a
declared non-empty-string shape. Fixed by typing the success case
non-empty-string.
===file===
<?php
/**
 * @param list<non-empty-string> $dirs
 * @return list<array{path: non-empty-string}>
 */
function resolvedPaths(array $dirs): array {
    $result = [];

    foreach ($dirs as $dir) {
        $path = realpath($dir);
        $result[] = ['path' => $path !== false ? $path : $dir];
    }

    return $result;
}
===expect===
