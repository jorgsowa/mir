===description===
An aliased class resolves to the original class at the usage site.
===cursor===
symbol
===file:src/Greeter.php===
<?php
namespace App;

final class Greeter {}
===file:main.php===
<?php
use App\Greeter as G;

$g = new <CURSOR>G();
===expect===
kind: class App\Greeter
type: App\Greeter
