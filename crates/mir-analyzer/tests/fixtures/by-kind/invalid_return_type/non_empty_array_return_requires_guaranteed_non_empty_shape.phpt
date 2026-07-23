===description===
A shaped array return value must be guaranteed non-empty (a required
property, or an open shape that may hold unknown extra keys) to satisfy a
declared non-empty-array/non-empty-list return type. A closed shape with no
required property — including the fully-empty `[]` case — is rejected.
===config===
suppress=UnusedParam
===file===
<?php

/** @return non-empty-array<string, int> */
function returns_empty_array(): array {
    return [];
}

/**
 * @param array{a?: int} $x
 * @return non-empty-array<string, int>
 */
function returns_closed_shape_all_optional(array $x): array {
    return $x;
}

/** @return non-empty-array<string, int> */
function returns_shape_with_required_prop(): array {
    return ['a' => 1];
}

/**
 * @param array{...} $x
 * @return non-empty-array<string, int>
 */
function returns_open_shape(array $x): array {
    return $x;
}
===expect===
InvalidReturnType@5:4-5:14: Return type 'array{}' is not compatible with declared 'non-empty-array<string, int>'
InvalidReturnType@13:4-13:14: Return type 'array{'a'?: int}' is not compatible with declared 'non-empty-array<string, int>'
