===source===
<?php
class A extends A {}
===expect===
CircularInheritance: class A extends A {}
