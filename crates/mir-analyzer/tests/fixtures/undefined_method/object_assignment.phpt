===description===
objectAssignment
===file===
<?php
class A {}
(new A)["b"] = 1;
===expect===
UndefinedMethod
===ignore===
TODO
