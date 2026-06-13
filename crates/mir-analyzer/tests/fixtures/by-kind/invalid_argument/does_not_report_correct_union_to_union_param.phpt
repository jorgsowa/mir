===description===
does not report correct union to union param
===config===
suppress=ForbiddenCode
===file===
<?php
function f(string|int $x): void { var_dump($x); }
function test(): void { f('hello'); }
===expect===
