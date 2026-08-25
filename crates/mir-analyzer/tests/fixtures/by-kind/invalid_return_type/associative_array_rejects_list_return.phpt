===description===
Phan's `associative-array` is distinct from a list, so returning a plain list
does not satisfy the declaration.
===file===
<?php
/**
 * @return associative-array<string, int>
 */
function buildAssociativeArray() {
    return [1, 2, 3];
}
===expect===
InvalidReturnType@6:4-6:21: Return type 'array{0: 1, 1: 2, 2: 3}' is not compatible with declared 'array<string, int>&array{}'
