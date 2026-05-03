===description===
class self extends
===file===
<?php
class A extends A {}
===expect===
CircularInheritance: Class A has a circular inheritance chain
===ignore===
TODO
