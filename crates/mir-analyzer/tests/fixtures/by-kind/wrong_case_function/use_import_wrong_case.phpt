===description===
A use function import with wrong function name casing is reported.
===file:Lib.php===
<?php

namespace Lib;

function myFunc(): void {}
===file:Main.php===
<?php

use function Lib\MYFUNC;
//           ^^^^^^^^^^ WrongCaseFunction: Function name 'MYFUNC' has incorrect casing; use 'myFunc'

MYFUNC();
//<^^^^^^ WrongCaseFunction: Function name 'MYFUNC' has incorrect casing; use 'myFunc'
===expect===
