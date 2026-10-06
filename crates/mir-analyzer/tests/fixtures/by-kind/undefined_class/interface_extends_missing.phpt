===description===
interface extends missing
===file===
<?php
interface MyInterface extends MissingParentInterface {}
//                            ^^^^^^^^^^^^^^^^^^^^^^ UndefinedClass: Class MissingParentInterface does not exist
