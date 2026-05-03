===description===
defined before if no error
===file===
<?php
function foo(bool $c): string {
    $r = 'default';
    if ($c) { $r = 'hello'; }
    return $r;
}
===expect===
===ignore===
TODO
