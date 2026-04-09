===source===
<?php
function foo(): bool {
    $val = null;
    return isset($val);
}
===expect===
