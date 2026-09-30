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
//  ^^^^^^^^^^^ WrongCaseClass: Class name 'myClass' has incorrect casing; use 'MyClass'

$x = new myClass();
//       ^^^^^^^ WrongCaseClass: Class name 'myClass' has incorrect casing; use 'MyClass'
===expect===
