===description===
does not report mixed arg
===file===
<?php
function f(int $x): void { var_dump($x); }
function test(mixed $v): void { f($v); }
===expect===
===ignore===
TODO
