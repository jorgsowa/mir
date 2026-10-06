===description===
reports function when variable callable not statically resolvable
===file===
<?php
function helper(): void {}
//<^^^^^^^^^^^^^^^^^^^^^^^^^^ UnusedFunction: Function helper() is never called

$fn = 'helper';
call_user_func($fn);
