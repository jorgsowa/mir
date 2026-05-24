===description===
undefinedParentClass
===file===
<?php
/**
 * @psalm-suppress UndefinedClass
 */
class B extends A {}

$b = new B();
===expect===
MissingDependency
===ignore===
TODO
