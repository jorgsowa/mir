===description===
class self extends
===file===
<?php
class A extends A {}
===expect===
CircularInheritance@2:0-2:20: Class A has a circular inheritance chain
