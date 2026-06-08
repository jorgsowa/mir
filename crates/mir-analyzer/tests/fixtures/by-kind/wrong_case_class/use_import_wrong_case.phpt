===description===
A use import with wrong class name casing is reported.
===file:Lib.php===
<?php

namespace Lib;

class MyClass {}
===file:Main.php===
<?php

use Lib\myClass;

new myClass();
===expect===
Main.php: WrongCaseClass@3:5-3:16: Class name 'myClass' has incorrect casing; use 'MyClass'
Main.php: WrongCaseClass@5:5-5:12: Class name 'myClass' has incorrect casing; use 'MyClass'
