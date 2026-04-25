===file===
<?php
function f(): int {
    $x = json_decode('{}');
    return $x;
}
===expect===
