===description===
A use import with wrong class name casing is reported.
===config===
suppress=UnusedVariable
===file:Lib.php===
<?php

namespace Lib;

class MyClass {}
===file:Main.php===
<?php

use Lib\myClass;

$x = new myClass();
===expect===
Main.php: WrongCaseClass@3:5-3:16: Class name 'myClass' has incorrect casing; use 'MyClass'
Main.php: WrongCaseClass@5:10-5:17: Class name 'myClass' has incorrect casing; use 'MyClass'
