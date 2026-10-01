===description===
A namespaced `const` declaration name resolves to itself.
===cursor===
definition
===file===
<?php
namespace App;
const MAX_<CURSOR>SIZE = 100;
echo MAX_SIZE;
===expect===
test.php@3:0-3:21
