===description===
Negative control: a key missing from every closed shape of a union is still an error.
===file===
<?php
/** @param array{a: int}|array{b: string} $x */
function all_lack(array $x): void {
    echo $x['c'];
//          ^^^ NonExistentArrayOffset: Array offset 'c' does not exist
}

function branches(bool $c): void {
    $o = $c ? ['a' => 1] : ['b' => 2];
    echo $o['c'];
//          ^^^ NonExistentArrayOffset: Array offset 'c' does not exist
}
