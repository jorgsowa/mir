===description===
Negative control for the array-index-write auto-vivification fix: reading
a nullable array WITHOUT ever writing to it first must still flag — the
fix only strips null as a consequence of an actual write, not as a
blanket relaxation of array nullability.
===file===
<?php
function f(?array $data): array {
    return $data;
}
===expect===
NullableReturnStatement@3:4-3:17: Return type 'array|null' is not compatible with declared 'array'
