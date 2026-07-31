===description===
extension class via use alias
===config===
suppress=UnusedParam,UnusedFunction
===file===
<?php
use Swoole\Coroutine;
function f(Coroutine $x): void {}
===expect===
UndefinedClass@3:11-3:20: Class Swoole\Coroutine does not exist
