===source===
<?php
class A extends A {}
===expect===
CircularInheritance: Class A has a circular inheritance chain
