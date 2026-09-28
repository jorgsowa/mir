===description===
Moving a class to another namespace breaks the unchanged importer.
===file:Lib.php===
<?php
namespace Lib;
class Box {}
===file:Use.php===
<?php
namespace App;
use Lib\Box;
function run(): void { new Box(); }
===expect===
===edit:Lib.php===
<?php
namespace Other;
class Box {}
===expect===
Use.php: UndefinedClass@4:27-4:30: Class Lib\Box does not exist
