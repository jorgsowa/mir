===description===
undefinedParentClass
===file===
<?php
/**
 * @suppress UndefinedClass
 */
class B extends A {}

$b = new B();
===expect===
MissingDependency
===ignore===
TODO
