===description===
InvalidExtendClass fires only for the direct extends of a final class; a class extending a non-final intermediate is not flagged even if a grandparent is final.
===file===
<?php
final class Base {}
class Middle extends Base {}
class Child extends Middle {}
===expect===
InvalidExtendClass@3:0-3:28: Class Middle cannot extend final class Base
