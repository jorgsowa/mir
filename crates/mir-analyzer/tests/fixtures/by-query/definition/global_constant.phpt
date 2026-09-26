===description===
Go-to-definition on a user-defined global constant lands on its `const` declaration.
===ignore===
===cursor===
definition
===file===
<?php
const LIMIT = 10;
echo LIM<CURSOR>IT;
===expect===
test.php@2:0-2:17
