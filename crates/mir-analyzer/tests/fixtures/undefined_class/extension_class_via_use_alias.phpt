===description===
extension class via use alias
===file===
<?php
use ast\Node;
function f(Node $x): void {}
===expect===
UndefinedClass@3:12: Class ast\Node does not exist
UnusedParam@3:12: Parameter $x is never used
