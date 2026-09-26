===description===
Go-to-definition on a built-in function lands in the bundled stubs.
===cursor===
definition
===file===
<?php
echo str<CURSOR>len('abc');
===expect===
stubs/Core/Core.php@57:0-57:39
