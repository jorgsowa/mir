===source===
<?php
function f(string|int $x): void { var_dump($x); }
function test(): void { f('hello'); }
===expect===
