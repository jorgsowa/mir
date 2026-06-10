===description===
No other string allowed for value of keyed array
===ignore===
TODO
===file===
<?php
/**
 * @return value-of<array{a: "foo", b: "bar"}>
 */
function getValue() {
    return "adams";
}

===expect===
