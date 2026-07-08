===description===
FP: PHP canonicalizes a numeric string array key to an int key at runtime —
`['0' => 'x']` and `$arr[0]` are the same slot — but mir kept the literal
string key un-canonicalized, so accessing it via an int-literal index
wrongly reported NonExistentArrayOffset.
===file===
<?php
function f(): void {
    $arr = ['0' => 'x', '1' => 'y'];
    echo $arr[0];
}
===expect===
