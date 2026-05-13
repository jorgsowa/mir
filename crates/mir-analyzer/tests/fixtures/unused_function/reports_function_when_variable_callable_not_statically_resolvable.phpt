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
UnusedFunction@2:0: Function helper() is never called
