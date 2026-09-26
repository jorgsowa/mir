===description===
Function names are case-insensitive, so a differently-cased call still resolves.
===cursor===
definition
===file===
<?php
function greet(): string { return 'hi'; }
echo GRE<CURSOR>ET();
===expect===
test.php@2:0-2:41
