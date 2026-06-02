===description===
Circular reference
===file===
<?php
class A extends A {}
===expect===
CircularInheritance@2:0-2:20: Class A has a circular inheritance chain
