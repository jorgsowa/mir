===description===
Wrong case parent class name in extends is reported.
===file===
<?php
class Base {}
class Child extends base {}
===expect===
WrongCaseClass@3:0-3:27: Class name 'base' has incorrect casing; use 'Base'
