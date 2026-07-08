===description===
"01" is NOT a canonical PHP integer string (leading zero) — it stays a
string key, so accessing it via the int literal `0` correctly still
reports NonExistentArrayOffset.
===file===
<?php
function f(): void {
    $arr = ['01' => 'x'];
    echo $arr[0];
}
===expect===
NonExistentArrayOffset@4:14-4:15: Array offset '0' does not exist
