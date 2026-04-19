===source===
<?php
use ast\Node;
function f(Node $x): void {}
===expect===
UnusedParam: Parameter $x is never used
UndefinedClass: Class ast\Node does not exist
