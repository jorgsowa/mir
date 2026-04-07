===source===
<?php
function f(int $x): void { var_dump($x); }
function test(): void { f('hello'); }
===expect===
InvalidArgument at 3:26
