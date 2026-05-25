===description===
does not report null passed to nullable param
===config===
suppress=UnusedParam,UnusedFunction
===file===
<?php
function f(?string $x): void {}
function test(): void { f(null); }
===expect===
