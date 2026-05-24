===description===
interface extends missing
===file===
<?php
interface MyInterface extends MissingParentInterface {}
===expect===
UndefinedClass@2:31: Class MissingParentInterface does not exist
