===description===
InvalidExtendClass fires for each class in a chain of final-extends: both Middle (extends final Base) and Child (extends final Middle) are flagged.
===file===
<?php
final class Base {}
final class Middle extends Base {}
class Child extends Middle {}
===expect===
InvalidExtendClass@3:6-3:34: Class Middle cannot extend final class Base
InvalidExtendClass@4:0-4:29: Class Child cannot extend final class Middle
