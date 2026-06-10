===description===
Explicit variable key
===ignore===
TODO
===file===
<?php
/** @param list<int> $a */
function takesList($a): void {}

/** @return array-key */
function getKey() {
    return 0;
}

$a = [getKey() => 1];
takesList($a);
===expect===
