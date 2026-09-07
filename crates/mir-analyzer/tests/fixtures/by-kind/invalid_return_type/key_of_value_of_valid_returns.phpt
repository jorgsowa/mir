===description===
key-of and value-of resolve to the real key/value types; valid returns are accepted
===file===
<?php
/** @return key-of<list<string>> */
function getKey() {
    return 0;
}

/** @return value-of<array{a: "foo", b: "bar"}> */
function getValue() {
    return "foo";
}

/** @return value-of<array<int, string>> */
function getStr() {
    return "anything";
}

===expect===
