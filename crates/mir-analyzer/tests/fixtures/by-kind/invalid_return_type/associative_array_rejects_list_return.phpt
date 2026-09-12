===description===
`associative-array` return types reject lists.
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
