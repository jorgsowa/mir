===description===
both branches no error
===file===
<?php
function foo(bool $c): string {
    if ($c) { $r = 'a'; } else { $r = 'b'; }
    return $r;
}
===expect===
