===description===
class three cycle
===file===
<?php
class A extends B {}
class B extends C {}
class C extends A {}
===expect===
CircularInheritance@4:0: Class C has a circular inheritance chain
