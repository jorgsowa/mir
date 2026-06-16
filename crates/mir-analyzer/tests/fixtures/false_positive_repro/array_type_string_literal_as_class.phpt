===description===
FALSE POSITIVE reproducer. Valid PHP: The string literal `'string[]'` is a runtime value, not a class name.
mir 0.42.0 currently emits (the bug): UndefinedClass@6:23-6:33 (string[]) + InvalidArgument@6:23-6:33 (expected class-string)
Expected: no issue. Remove ===ignore=== to activate once fixed.
===ignore===
===config===
php_version=8.4
===file===
<?php
/** @param class-string $type */
function deserialize(string $type, string $payload): mixed { return null; }
function run(string $payload): mixed {
    // expect: UndefinedClass "string[]" (array-type string parsed as a class)
    return deserialize('string[]', $payload);
}
===expect===
