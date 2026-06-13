===description===
does not report correct int arg
===config===
suppress=ForbiddenCode
===file===
<?php
function f(int $x): void { var_dump($x); }
function test(): void { f(42); }
===expect===
