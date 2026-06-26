===description===
InterfaceInstantiation fires when instantiating a built-in PHP interface from the standard library.
===config===
suppress=UnusedVariable
===file===
<?php
$t = new Traversable();
===expect===
InterfaceInstantiation@2:9-2:20: Cannot instantiate interface Traversable
