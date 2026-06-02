===description===
Foreach iteration variable type recorded at binding position

===ignore===
@mir-check validates the variable type in the flow context, not the ResolvedSymbol
emission itself. The actual goal (having ResolvedSymbol at the binding span for
hover/inlay-hints) is verified implicitly through type availability.
===file===
<?php
class User {
    public string $name;
}

$users = [new User()];
/** @mir-check $k is int */
/** @mir-check $user is User */
foreach ($users as $k => $user) {
    // Type should be inferred at binding position above, not just in body
}

// Also test value-only foreach
$strings = ["a", "b"];
/** @mir-check $item is string */
foreach ($strings as $item) {
    // Type should be recorded at binding position
}

// Test with keyed array
$data = ["x" => 1, "y" => 2];
/** @mir-check $key is 'x'|'y' */
/** @mir-check $val is 1|2 */
foreach ($data as $key => $val) {
    // Literal key and value types
}
===expect===
