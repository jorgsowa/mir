===description===
Phan's `associative-array` accepts a definitely non-list returned array with
matching string keys and value types.
===file===
<?php
/**
 * @return associative-array<string, int>
 */
function buildAssociativeArray() {
    return ['x' => 1, 'y' => 2];
}
===expect===
