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
