===description===
Returning a shape array with incompatible value types for array<int, int>
should be flagged as InvalidReturnType.
===file===
<?php
/** @return array<int, int> */
function test(): array {
    return ['a' => 'hello'];
//  ^^^^^^^^^^^^^^^^^^^^^^^^ InvalidReturnType: Return type 'array{'a': "hello"}' is not compatible with declared 'array<int, int>'
}
