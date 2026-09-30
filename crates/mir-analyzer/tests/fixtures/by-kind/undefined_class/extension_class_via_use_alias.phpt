===description===
extension class via use alias
===config===
suppress=UnusedParam,UnusedFunction
===file===
<?php
use Swoole\Coroutine;
function f(Coroutine $x): void {}
//         ^^^^^^^^^ UndefinedClass: Class Swoole\Coroutine does not exist
===expect===
