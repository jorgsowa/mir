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
InvalidReturnType@6:4-6:19: Return type '"adams"' is not compatible with declared '"foo"|"bar"'
