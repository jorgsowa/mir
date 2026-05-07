===description===
class two cycle
===file===
<?php
class A extends B {}
class B extends A {}
===expect===
CircularInheritance@3:0: Class B has a circular inheritance chain
