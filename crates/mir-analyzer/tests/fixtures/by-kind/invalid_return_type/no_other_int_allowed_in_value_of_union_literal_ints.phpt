===description===
No other int allowed in value of union literal ints
===file===
<?php
/**
 * @return value-of<list<0|1|2>|array{0: 3, 1: 4}>
 */
function getValue() {
    return 5;
}

===expect===
UndefinedDocblockClass@5:10-5:18: Docblock type 'value-of' does not exist
