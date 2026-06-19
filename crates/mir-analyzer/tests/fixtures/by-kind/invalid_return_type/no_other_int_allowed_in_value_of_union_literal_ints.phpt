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
InvalidReturnType@6:4-6:13: Return type '5' is not compatible with declared '0|1|2|3|4'
