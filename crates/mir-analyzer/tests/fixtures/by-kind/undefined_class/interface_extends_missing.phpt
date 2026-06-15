===description===
interface extends missing
===file===
<?php
interface MyInterface extends MissingParentInterface {}
===expect===
UndefinedClass@2:30-2:52: Class MissingParentInterface does not exist
