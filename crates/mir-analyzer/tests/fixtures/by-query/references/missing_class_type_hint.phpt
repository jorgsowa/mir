===description===
References to a missing class from a type hint still resolve a name and find sibling hints.
===cursor===
references
===file===
<?php
function f(Miss<CURSOR>ing $m): void {}
function g(Missing $m): void {}
===expect===
test.php@2:11-2:18
test.php@3:11-3:18
