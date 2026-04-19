===source===
<?php
// Bug: foreach over a keyed array always produced TMixed for the key type.
// With TMixed, no InvalidArgument is reported even when the key is provably
// string and the parameter expects int. After the fix, the key is typed as
// the literal string keys and the mismatch is caught.
function takes_int(int $k): void { var_dump($k); }

function foo(): void {
    $arr = ['hello' => 1, 'world' => 2];
    foreach ($arr as $k => $v) {
        takes_int($k);
    }
}
===expect===
InvalidArgument: Argument $k of takes_int() expects 'int', got '"hello"|"world"'
