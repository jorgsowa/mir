===description===
class self extends
===file===
<?php
class A extends A {}
//<^^^^^^^^^^^^^^^^^^^^ CircularInheritance: Class A has a circular inheritance chain
===expect===
