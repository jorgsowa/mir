===description===
Go-to-definition resolves a class that only PSR-4 autoloading can find.
===cursor===
definition
===file:composer.json===
{"autoload":{"psr-4":{"App\\":"src/"}}}
===file:src/Greeter.php===
<?php
namespace App;

final class Greeter {}
===file:main.php===
<?php
$g = new \App\Gree<CURSOR>ter();
===expect===
src/Greeter.php@4:6-4:22
