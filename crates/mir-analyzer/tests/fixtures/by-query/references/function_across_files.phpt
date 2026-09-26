===cursor===
references
===file:helpers.php===
<?php
function helper(): int { return 1; }
===file:main.php===
<?php
echo hel<CURSOR>per();
echo helper() + helper();
===expect===
main.php@2:5-2:11
main.php@3:5-3:11
main.php@3:16-3:22
