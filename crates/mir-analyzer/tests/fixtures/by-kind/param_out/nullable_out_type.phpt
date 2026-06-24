===description===
@param-out with a nullable type: after the call the variable is nullable.
===config===
suppress=UnusedVariable,UnusedFunction,MixedAssignment
===file===
<?php
/**
 * @param-out int|null $found
 */
function findFirst(array $haystack, mixed &$found): bool {
    $found = $haystack[0] ?? null;
    return $found !== null;
}

findFirst([1, 2], $val);
/** @mir-check $val is int|null */
$_ = $val;
===expect===
