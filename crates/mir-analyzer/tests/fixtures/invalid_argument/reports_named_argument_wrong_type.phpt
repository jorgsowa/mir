===source===
<?php
function f(int $x): void { var_dump($x); }
function test(): void { f(x: 'hello'); }
===expect===
InvalidArgument: x: 'hello'
