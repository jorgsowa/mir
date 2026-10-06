===description===
Wrong case parent class name in extends is reported.
===file===
<?php
class Base {}
class Child extends base {}
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^ WrongCaseClass: Class name 'base' has incorrect casing; use 'Base'
