===description===
abstractClassInstantiation
===file===
<?php
abstract class A {}
new A();
===expect===
AbstractInstantiation@3:4: Cannot instantiate abstract class A
