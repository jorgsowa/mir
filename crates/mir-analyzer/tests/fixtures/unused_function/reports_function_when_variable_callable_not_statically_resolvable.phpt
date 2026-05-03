===description===
reports function when variable callable not statically resolvable
===config===
find_dead_code=true
===file===
<?php
function helper(): void {}

$fn = 'helper';
call_user_func($fn);
===expect===
# Variable-based callable cannot be statically resolved — helper is still flagged
UnusedFunction: Function helper() is never called
===ignore===
TODO
