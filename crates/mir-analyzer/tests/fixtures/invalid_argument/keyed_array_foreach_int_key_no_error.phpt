===file===
<?php
// Bug: foreach over a list-shaped keyed array always produced TMixed for the key.
// A function expecting int should not receive an InvalidArgument when the array
// has only integer keys.
function takes_int(int $k): void { var_dump($k); }

function foo(): void {
    $arr = [0 => 'a', 1 => 'b', 2 => 'c'];
    foreach ($arr as $k => $v) {
        takes_int($k);
    }
}
===expect===
