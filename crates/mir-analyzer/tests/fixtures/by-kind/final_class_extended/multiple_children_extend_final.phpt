===description===
InvalidExtendClass fires separately for each class that extends the same final class.
===file===
<?php
final class Base {}
class ChildA extends Base {}
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidExtendClass: Class ChildA cannot extend final class Base
class ChildB extends Base {}
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidExtendClass: Class ChildB cannot extend final class Base
