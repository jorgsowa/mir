===file===
<?php
class A extends B {}
class B extends A {}
===expect===
CircularInheritance: Class B has a circular inheritance chain
