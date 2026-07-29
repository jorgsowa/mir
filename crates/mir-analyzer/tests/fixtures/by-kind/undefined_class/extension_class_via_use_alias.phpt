===description===
extension class via use alias
===config===
suppress=UnusedParam,UnusedFunction
===file===
<?php
use MongoDB\BSON\Binary;
function f(Binary $x): void {}
===expect===
UndefinedClass@3:11-3:17: Class MongoDB\BSON\Binary does not exist
