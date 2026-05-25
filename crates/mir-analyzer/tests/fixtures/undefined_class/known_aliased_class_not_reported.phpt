===description===
known aliased class not reported
===config===
suppress=UnusedParam,UnusedFunction
===file===
<?php
class Bar {}
use Bar as Baz;
function f(Baz $x): void {}
===expect===
