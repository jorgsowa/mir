===file===
<?php
function f(mixed $x): void {}
function test(): void { f(null); }
===expect===
UnusedParam: Parameter $x is never used
