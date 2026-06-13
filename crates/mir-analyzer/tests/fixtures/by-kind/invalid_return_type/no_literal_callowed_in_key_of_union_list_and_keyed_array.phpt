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
UndefinedDocblockClass@5:10-5:16: Docblock type 'key-of' does not exist
