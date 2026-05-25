===description===
does not report null passed to mixed param
===config===
suppress=UnusedParam,UnusedFunction
===file===
<?php
function f(mixed $x): void {}
function test(): void { f(null); }
===expect===
