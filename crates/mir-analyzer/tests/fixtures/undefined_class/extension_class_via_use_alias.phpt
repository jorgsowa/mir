===source===
<?php
use ast\Node;
function f(Node $x): void {}
===expect===
UnusedParam: $x
UndefinedClass: Node
