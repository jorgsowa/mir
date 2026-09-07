===description===
FALSE POSITIVE reproducer. Valid PHP: The string literal `'string[]'` is a runtime value, not a class name.
Expected: no issue.
===config===
php_version=8.4
suppress=UnusedParam
===file===
<?php
/** @param class-string $type */
function deserialize(string $type, string $payload): mixed { return null; }
function run(string $payload): mixed {
    // expect: UndefinedClass "string[]" (array-type string parsed as a class)
    return deserialize('string[]', $payload);
}
===expect===
