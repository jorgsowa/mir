===source===
<?php
interface MyInterface extends MissingParentInterface {}
===expect===
UndefinedClass: MissingParentInterface
