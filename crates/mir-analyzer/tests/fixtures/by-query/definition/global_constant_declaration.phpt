===description===
Go-to-definition on a global `const` declaration name resolves to the declaration itself.
===cursor===
definition
===file===
<?php
const MAX_<CURSOR>SIZE = 100;
echo MAX_SIZE;
===expect===
test.php@2:0-2:21
