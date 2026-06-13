===description===
ClassName::class inside class_exists() arg does not emit UndefinedClass
===config===
suppress=UnusedVariable
===file===
<?php
$exists = class_exists(\Optional\Pkg::class);
===expect===
