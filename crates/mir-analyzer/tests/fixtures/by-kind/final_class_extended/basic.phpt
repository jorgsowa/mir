===description===
Basic
===file===
<?php
final class Base {}
class Child extends Base {}
===expect===
InvalidExtendClass@3:0-3:27: Class Child cannot extend final class Base
