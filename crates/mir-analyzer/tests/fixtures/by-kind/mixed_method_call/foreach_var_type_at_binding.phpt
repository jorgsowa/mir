===description===
Foreach iteration variable type recorded at binding position
===config===
<mir>
  <issueHandlers>
    <UnusedForeachValue errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class User {
//<^^^^^^^^^^^^ MissingConstructor: Class User has uninitialized properties but no constructor
    public string $name;
}

$users = [new User()];
/** @mir-check $k is int */
/** @mir-check $user is User */
foreach ($users as $k => $user) {
//<^ +2:1 TypeCheckMismatch: Type of $user is expected to be User, got mixed
    // Type should be inferred at binding position above, not just in body
}

// Also test value-only foreach
$strings = ["a", "b"];
/** @mir-check $item is string */
foreach ($strings as $item) {
//<^ +2:1 TypeCheckMismatch: Type of $item is expected to be string, got mixed
    // Type should be recorded at binding position
}

// Test with keyed array
$data = ["x" => 1, "y" => 2];
/** @mir-check $key is 'x'|'y' */
/** @mir-check $val is 1|2 */
foreach ($data as $key => $val) {
//<^ +2:1 TypeCheckMismatch: Type of $val is expected to be 1|2, got mixed
    // Literal key and value types
}
===expect===
