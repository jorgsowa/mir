===description===
A function imported with `use function` lands on its declaration.
===cursor===
definition
===file:src/helpers.php===
<?php
namespace App\Support;

function helper(): int { return 1; }
===file:main.php===
<?php
use function App\Support\helper;

echo hel<CURSOR>per();
===expect===
src/helpers.php@4:0-4:36
