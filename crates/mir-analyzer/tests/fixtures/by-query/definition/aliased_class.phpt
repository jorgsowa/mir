===description===
A class used through a `use ... as` alias lands on the original class.
===cursor===
definition
===file:src/Greeter.php===
<?php
namespace App;

final class Greeter {}
===file:main.php===
<?php
use App\Greeter as G;

$g = new <CURSOR>G();
===expect===
src/Greeter.php@4:6-4:22
