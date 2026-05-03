===description===
does not report null passed to nullable param
===file===
<?php
function f(?string $x): void {}
function test(): void { f(null); }
===expect===
UnusedParam@2:11: Parameter $x is never used
===ignore===
TODO
