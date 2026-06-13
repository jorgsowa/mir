===description===
Plain string not resolved as class
===config===
suppress=UnusedVariable
===file===
<?php
// A plain string literal should NOT be resolved as a class name
// This should NOT emit UndefinedClass even though "NonExistentClass" is not defined
$className = "NonExistentClass";
$instance = new $className();
===expect===
