===description===
InvalidExtendClass fires for each class in a chain of final-extends: both Middle (extends final Base) and Child (extends final Middle) are flagged.
===file===
<?php
final class Base {}
final class Middle extends Base {}
//    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidExtendClass: Class Middle cannot extend final class Base
class Child extends Middle {}
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidExtendClass: Class Child cannot extend final class Middle
===expect===
