===description===
class two cycle
===file===
<?php
class A extends B {}
class B extends A {}
//<^^^^^^^^^^^^^^^^^^^^ CircularInheritance: Class B has a circular inheritance chain
===expect===
