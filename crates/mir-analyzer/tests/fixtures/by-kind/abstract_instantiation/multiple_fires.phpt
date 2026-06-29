===description===
Two separate abstract class instantiations each produce their own AbstractInstantiation diagnostic.
===file===
<?php
abstract class Alpha {}
abstract class Beta {}
new Alpha();
new Beta();
===expect===
AbstractInstantiation@4:4-4:9: Cannot instantiate abstract class Alpha
AbstractInstantiation@5:4-5:8: Cannot instantiate abstract class Beta
