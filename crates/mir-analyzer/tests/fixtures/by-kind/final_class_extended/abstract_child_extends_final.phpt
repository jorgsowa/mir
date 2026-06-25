===description===
InvalidExtendClass fires when an abstract class extends a final class.
===file===
<?php
final class Base {}
abstract class Child extends Base {}
===expect===
InvalidExtendClass@3:9-3:36: Class Child cannot extend final class Base
