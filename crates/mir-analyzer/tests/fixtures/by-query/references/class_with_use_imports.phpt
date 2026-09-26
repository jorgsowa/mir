===cursor===
references use_imports
===file:src/Greeter.php===
<?php
namespace App;

final class Greeter {}
===file:main.php===
<?php
use App\Greeter;

$g = new Gre<CURSOR>eter();
function make(): Greeter { return new Greeter(); }
===expect===
main.php@2:4-2:15
main.php@4:9-4:16
main.php@5:17-5:24
main.php@5:38-5:45
