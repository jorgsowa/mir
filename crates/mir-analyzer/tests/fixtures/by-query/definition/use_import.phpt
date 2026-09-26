===cursor===
definition
===file:src/Greeter.php===
<?php
namespace App;

final class Greeter {}
===file:main.php===
<?php
use App\Gree<CURSOR>ter;

$g = new Greeter();
===expect===
src/Greeter.php@4:6-4:22
