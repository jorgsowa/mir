===description===
A use function import with wrong function name casing is reported.
===file:Lib.php===
<?php

namespace Lib;

function myFunc(): void {}
===file:Main.php===
<?php

use function Lib\MYFUNC;

MYFUNC();
===expect===
Main.php: WrongCaseFunction@3:14-3:24: Function name 'MYFUNC' has incorrect casing; use 'myFunc'
Main.php: WrongCaseFunction@5:1-5:7: Function name 'MYFUNC' has incorrect casing; use 'myFunc'
