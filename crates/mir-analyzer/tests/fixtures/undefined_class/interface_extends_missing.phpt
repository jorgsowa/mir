===source===
<?php
interface MyInterface extends MissingParentInterface {}
===expect===
UndefinedClass: Class MissingParentInterface does not exist
