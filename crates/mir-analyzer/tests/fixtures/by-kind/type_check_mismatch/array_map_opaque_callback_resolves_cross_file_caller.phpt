===description===
The opaque-callback caller scan is workspace-wide, not file-local: a caller
in a different file must still resolve the callback's return type.
===config===
suppress=MixedAssignment
===file:lib.php===
<?php
function apply(callable $cb, array $nums): array {
    $result = array_map($cb, $nums);
    /** @mir-check $result is array<array-key, string> */
    return $result;
}
===file:app.php===
<?php
function useIt(array $nums): void {
    apply(fn(int $x): string => (string) $x, $nums);
}
===expect===
