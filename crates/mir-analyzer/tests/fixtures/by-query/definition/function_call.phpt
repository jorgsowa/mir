===description===
Go-to-definition on a function call lands on the function's name.
===cursor===
definition
===file===
<?php
function greet(): string { return 'hi'; }
echo gr<CURSOR>eet();
===expect===
test.php@2:0-2:41
