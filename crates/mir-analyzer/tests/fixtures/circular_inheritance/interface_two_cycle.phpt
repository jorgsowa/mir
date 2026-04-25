===file===
<?php
interface I1 extends I2 {}
interface I2 extends I1 {}
===expect===
CircularInheritance: Class I2 has a circular inheritance chain
