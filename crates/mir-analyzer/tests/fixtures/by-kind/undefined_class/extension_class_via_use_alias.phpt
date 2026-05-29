===description===
extension class via use alias
===config===
suppress=UnusedParam,UnusedFunction
===file===
<?php
use ast\Node;
function f(Node $x): void {}
===expect===
UndefinedClass@3:12-3:16: Class ast\Node does not exist
