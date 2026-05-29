===description===
class three cycle
===file===
<?php
class A extends B {}
class B extends C {}
class C extends A {}
===expect===
CircularInheritance@4:0-4:20: Class C has a circular inheritance chain
