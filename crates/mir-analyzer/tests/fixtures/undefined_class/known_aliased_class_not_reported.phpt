===description===
known aliased class not reported
===file===
<?php
class Bar {}
use Bar as Baz;
function f(Baz $x): void {}
===expect===
UnusedParam@4:11: Parameter $x is never used
