===description===
`associative-array` is treated as an `array` alias, so returning a list is
valid for the same declared return type.
===file===
<?php
/**
 * @return associative-array<array-key, int>
 */
function buildAssociativeArray(): array {
    return [1, 2, 3];
}
===expect===
