===description===
interface extends missing
===file===
<?php
interface MyInterface extends MissingParentInterface {}
===expect===
UndefinedClass@2:31-2:53: Class MissingParentInterface does not exist
