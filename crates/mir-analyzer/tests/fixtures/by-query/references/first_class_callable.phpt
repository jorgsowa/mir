===description===
A first-class callable counts as a function reference.
===cursor===
references
===file===
<?php
function helper(): int { return 1; }
$f = helper(...);
echo hel<CURSOR>per();
===expect===
test.php@3:5-3:11
test.php@4:5-4:11
