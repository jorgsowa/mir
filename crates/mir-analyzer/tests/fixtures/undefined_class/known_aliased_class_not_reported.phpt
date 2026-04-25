===file===
<?php
class Bar {}
use Bar as Baz;
function f(Baz $x): void {}
===expect===
UnusedParam: Parameter $x is never used
