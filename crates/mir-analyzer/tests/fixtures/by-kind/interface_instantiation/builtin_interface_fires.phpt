===description===
InterfaceInstantiation fires when instantiating a built-in PHP interface from the standard library.
===config===
suppress=UnusedVariable
===file===
<?php
$t = new Traversable();
//       ^^^^^^^^^^^ InterfaceInstantiation: Cannot instantiate interface Traversable
===expect===
