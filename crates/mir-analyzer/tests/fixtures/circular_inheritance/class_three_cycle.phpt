===source===
<?php
class A extends B {}
class B extends C {}
class C extends A {}
===expect===
CircularInheritance: Class C has a circular inheritance chain
