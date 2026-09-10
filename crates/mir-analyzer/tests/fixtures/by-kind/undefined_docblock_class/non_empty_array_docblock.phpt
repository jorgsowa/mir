===description===
`non-empty-array` resolves to the non-empty array pseudo-type as a
docblock keyword (it had a generic arm but was missing from the
keyword table, so the bare form parsed as a nonexistent class): an
empty array is rejected where a non-empty one is required, and a
non-empty one is accepted.
===config===
suppress=UnusedParam,UnusedVariable
===file===
<?php
/** @param non-empty-array $xs */
function useList(array $xs): void {
    /** @mir-check $xs is non-empty-array */
    $_ = 1;
}

/** @return non-empty-array */
function makeList(): array {
    return [1, 2];
}

/** @return non-empty-array */
function makeEmptyList(): array {
    return [];
}

useList([1]);
useList([]);

===expect===
InvalidReturnType@15:4-15:14: Return type 'array{}' is not compatible with declared 'non-empty-array'
InvalidArgument@19:8-19:10: Argument $xs of useList() expects 'non-empty-array', got 'array{}'
