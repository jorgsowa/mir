===description===
An unqualified built-in call inside a namespace lands on the global stub.
===cursor===
definition
===file===
<?php
namespace App;

echo str<CURSOR>len('abc');
===expect===
stubs/Core/Core.php@57:0-57:39
