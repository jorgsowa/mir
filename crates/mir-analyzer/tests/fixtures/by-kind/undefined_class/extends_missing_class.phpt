===description===
extends missing class
===file===
<?php
class Foo extends MissingBase {}
===expect===
UndefinedClass@2:18-2:29: Class MissingBase does not exist
