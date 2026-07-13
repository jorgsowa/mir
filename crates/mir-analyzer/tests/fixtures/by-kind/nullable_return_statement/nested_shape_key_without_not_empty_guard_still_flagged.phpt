===description===
Nested shape key access without a !empty() guard is still flagged nullable
===file===
<?php
/** @param array{a: array{b: ?string}} $x */
function f(array $x): string {
    return $x['a']['b'];
}
===expect===
NullableReturnStatement@4:4-4:24: Return type 'string|null' is not compatible with declared 'string'
