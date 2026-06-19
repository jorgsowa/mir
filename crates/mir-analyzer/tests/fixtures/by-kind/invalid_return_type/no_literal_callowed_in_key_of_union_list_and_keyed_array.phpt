===description===
No literal c allowed in key of union list and keyed array
===file===
<?php
/**
 * @return key-of<list<int>|array{a: int, b: int}>
 */
function getKey() {
    return "c";
}

===expect===
InvalidReturnType@6:4-6:15: Return type '"c"' is not compatible with declared 'int|"a"|"b"'
