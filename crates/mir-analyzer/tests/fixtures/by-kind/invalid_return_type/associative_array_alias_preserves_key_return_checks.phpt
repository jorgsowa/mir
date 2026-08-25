===description===
The alias still preserves explicit key constraints for return checking: a
string-keyed shape is not a valid `associative-array<int, int>`.
===file===
<?php
/**
 * @return associative-array<int, int>
 */
function buildIntKeyedAssociativeArray(): array {
    return ['x' => 1];
}
===expect===
InvalidReturnType@6:4-6:22: Return type 'array{'x': 1}' is not compatible with declared 'array<int, int>'
