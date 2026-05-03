===description===
isset check not reported
===file===
<?php
function foo(): bool {
    $val = null;
    return isset($val);
}
===expect===
===ignore===
TODO
