===description===
A class imported via group `use` lands on its declaration.
===cursor===
definition
===file:src/Greeter.php===
<?php
namespace App;

final class Greeter {}
===file:src/Other.php===
<?php
namespace App;

final class Other {}
===file:main.php===
<?php
use App\{Greeter, Other};

$g = new Gree<CURSOR>ter();
===expect===
src/Greeter.php@4:6-4:22
