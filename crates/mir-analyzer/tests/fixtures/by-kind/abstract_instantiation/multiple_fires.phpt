===description===
Two separate abstract class instantiations each produce their own AbstractInstantiation diagnostic.
===file===
<?php
abstract class Alpha {}
abstract class Beta {}
new Alpha();
//  ^^^^^ AbstractInstantiation: Cannot instantiate abstract class Alpha
new Beta();
//  ^^^^ AbstractInstantiation: Cannot instantiate abstract class Beta
===expect===
