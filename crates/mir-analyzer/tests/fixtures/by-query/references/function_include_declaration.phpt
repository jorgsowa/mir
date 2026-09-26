===description===
With include_declaration, the declaration's name is listed alongside the usages.
===cursor===
references include_declaration
===file===
<?php
function helper(): int { return 1; }
echo hel<CURSOR>per();
===expect===
test.php@2:9-2:15
test.php@3:5-3:11
