===description===
A literal shape array is a non-empty-array and should satisfy non-empty-array<K,V>
===file===
<?php
/** @return non-empty-array<string, int> */
function test(): array {
    return ['a' => 1, 'b' => 2];
}
===expect===

