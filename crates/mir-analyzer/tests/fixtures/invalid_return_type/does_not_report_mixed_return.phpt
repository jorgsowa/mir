===description===
does not report mixed return
===file===
<?php
function f(): int {
    $x = json_decode('{}');
    return $x;
}
===expect===
===ignore===
TODO
