===description===
Abstract class instantiation
===file===
<?php
abstract class A {}
new A();
===expect===
AbstractInstantiation@3:5: Cannot instantiate abstract class A
