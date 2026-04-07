===source===
<?php
class Bar {}
use Bar as Baz;
function f(Baz $x): void {}
===expect===
# UnusedParam location bug: parameter $x is reported at 1:0 instead of the correct line
UnusedParam at 1:0
