===cursor===
references
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
main.php@4:9-4:16
main.php@5:17-5:24
main.php@5:38-5:45
