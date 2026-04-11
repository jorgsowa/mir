===source===
<?php
class A extends B {}
class B extends A {}
===expect===
CircularInheritance: class B extends A {}
