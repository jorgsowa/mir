===description===
InvalidExtendClass fires when an abstract class extends a final class.
===file===
<?php
final class Base {}
abstract class Child extends Base {}
//       ^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidExtendClass: Class Child cannot extend final class Base
===expect===
