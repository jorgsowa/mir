===description===
No other string allowed for value of keyed array
===file===
<?php
/**
 * @return value-of<array{a: "foo", b: "bar"}>
 */
function getValue() {
    return "adams";
}

===expect===
UndefinedDocblockClass@5:10-5:18: Docblock type 'value-of' does not exist
