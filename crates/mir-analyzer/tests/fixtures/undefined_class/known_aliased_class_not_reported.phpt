===source===
<?php
class Bar {}
use Bar as Baz;
function f(Baz $x): void {}
===expect===
UnusedParam: $x
