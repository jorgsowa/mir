===description===
InvalidExtendClass fires separately for each class that extends the same final class.
===file===
<?php
final class Base {}
class ChildA extends Base {}
class ChildB extends Base {}
===expect===
InvalidExtendClass@3:0-3:28: Class ChildA cannot extend final class Base
InvalidExtendClass@4:0-4:28: Class ChildB cannot extend final class Base
