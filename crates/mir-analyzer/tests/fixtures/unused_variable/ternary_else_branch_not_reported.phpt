===description===
ternary else branch not reported
===file===
<?php
function test(bool $flag): string {
    $default = 'fallback';
    return $flag ? 'yes' : $default;
}
===expect===
===ignore===
TODO
