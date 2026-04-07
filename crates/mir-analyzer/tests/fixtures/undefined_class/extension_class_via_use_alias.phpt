===source===
<?php
use ast\Node;
function f(Node $x): void {}
===expect===
UndefinedClass at 3:11
# UnusedParam location bug: parameter $x is reported at 1:0 instead of the correct line
UnusedParam at 1:0
