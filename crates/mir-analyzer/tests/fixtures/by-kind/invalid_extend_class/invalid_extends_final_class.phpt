===description===
Invalid extends final class
===file===
<?php

final class A {}

class B extends A {}
//<^^^^^^^^^^^^^^^^^^^^ InvalidExtendClass: Class B cannot extend final class A

===expect===
