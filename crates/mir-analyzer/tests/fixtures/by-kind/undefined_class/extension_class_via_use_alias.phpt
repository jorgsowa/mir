===description===
extension class via use alias
===config===
suppress=UnusedParam,UnusedFunction
===file===
<?php
use Ds\Vector;
function f(Vector $x): void {}
===expect===
UndefinedClass@3:11-3:17: Class Ds\Vector does not exist
