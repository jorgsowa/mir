===file===
<?php
// Bug: foreach over a keyed array (array shape) always produced TMixed for the key
// type instead of the actual key types. A function expecting string should not
// receive an InvalidArgument when iterating over a shape with string keys.
function takes_string(string $k): void { var_dump($k); }

function foo(): void {
    $arr = ['hello' => 1, 'world' => 2];
    foreach ($arr as $k => $v) {
        takes_string($k);
    }
}
===expect===
