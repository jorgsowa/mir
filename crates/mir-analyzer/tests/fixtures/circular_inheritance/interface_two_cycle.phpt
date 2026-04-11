===source===
<?php
interface I1 extends I2 {}
interface I2 extends I1 {}
===expect===
CircularInheritance: interface I2 extends I1 {}
