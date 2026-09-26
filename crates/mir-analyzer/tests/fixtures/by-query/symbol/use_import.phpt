===cursor===
symbol
===file:src/Greeter.php===
<?php
namespace App;

final class Greeter {}
===file:main.php===
<?php
use App\Gree<CURSOR>ter;

$g = new Greeter();
===expect===
kind: use import of class App\Greeter
type: class-string
