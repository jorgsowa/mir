===description===
interface extends missing
===file===
<?php
interface MyInterface extends MissingParentInterface {}
===expect===
UndefinedClass: Class MissingParentInterface does not exist
===ignore===
TODO
