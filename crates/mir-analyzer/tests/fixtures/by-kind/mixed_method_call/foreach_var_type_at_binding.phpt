===description===
Foreach iteration variable type recorded at binding position
===config===
suppress=UnusedForeachValue
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
MissingConstructor@2:0-2:12: Class User has uninitialized properties but no constructor
TypeCheckMismatch@9:1-11:2: Type of $user is expected to be User, got mixed
TypeCheckMismatch@16:1-18:2: Type of $item is expected to be string, got mixed
TypeCheckMismatch@24:1-26:2: Type of $val is expected to be 1|2, got mixed
