===description===
Undefined parent class
===config===
suppress=UnusedVariable
===file===
<?php
/**
 * @suppress UndefinedClass
 */
class B extends A {}

$b = new B();
===expect===
