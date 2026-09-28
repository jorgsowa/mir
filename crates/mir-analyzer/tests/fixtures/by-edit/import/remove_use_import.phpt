===description===
Dropping a `use` import makes the short name resolve to the current namespace.
===file:Lib.php===
<?php
namespace Lib;
class Box {}
===file:Use.php===
<?php
namespace App;
use Lib\Box;
function run(): void { new Box(); }
===edit:Use.php===
<?php
namespace App;
function run(): void { new Box(); }
===expect===
Use.php: UndefinedClass@3:27-3:30: Class App\Box does not exist
