===description===
Deprecated interface in generics
===ignore===
TODO
===file===
<?php
/** @deprecated */
interface MyInterface {}

/** @extends ArrayObject<array-key, MyInterface> */
class MyClass extends ArrayObject {}

===expect===
