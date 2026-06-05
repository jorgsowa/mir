===description===
A use import with correct class name casing is not reported.
===file:Lib.php===
<?php

namespace Lib;

class MyClass {}
===file:Main.php===
<?php

use Lib\MyClass;

$x = new MyClass();
===expect===
