===description===
Deprecated interface in generics
===file===
<?php
/** @deprecated */
interface MyInterface {}

/** @extends ArrayObject<array-key, MyInterface> */
class MyClass extends ArrayObject {}

===expect===
DeprecatedInterface
===ignore===
TODO
