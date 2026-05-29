===description===
extends missing class
===file===
<?php
class Foo extends MissingBase {}
===expect===
UndefinedClass@2:19-2:30: Class MissingBase does not exist
