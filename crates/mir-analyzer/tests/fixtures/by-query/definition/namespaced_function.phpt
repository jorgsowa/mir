===description===
A call to a namespaced function lands on its declaration in another file.
===cursor===
definition
===file:src/helpers.php===
<?php
namespace App;

function helper(): int { return 1; }
===file:main.php===
<?php
namespace App;

echo hel<CURSOR>per();
===expect===
src/helpers.php@4:0-4:36
