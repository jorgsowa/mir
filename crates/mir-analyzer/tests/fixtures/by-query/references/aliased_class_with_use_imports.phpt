===description===
References through an alias resolve to the original class, including the aliased import.
===cursor===
references use_imports
===file:src/Greeter.php===
<?php
namespace App;

final class Greeter {}
===file:main.php===
<?php
use App\Greeter as G;

$a = new <CURSOR>G();
$b = new \App\Greeter();
===expect===
main.php@2:4-2:20
main.php@4:9-4:10
main.php@5:9-5:21
