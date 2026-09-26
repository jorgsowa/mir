===description===
Differently-cased calls reference the same function.
===cursor===
references
===file===
<?php
function helper(): int { return 1; }
echo hel<CURSOR>per();
echo HELPER();
echo Helper();
===expect===
test.php@3:5-3:11
test.php@4:5-4:11
test.php@5:5-5:11
