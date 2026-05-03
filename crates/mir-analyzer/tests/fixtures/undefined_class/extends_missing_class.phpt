===description===
extends missing class
===file===
<?php
class Foo extends MissingBase {}
===expect===
UndefinedClass: Class MissingBase does not exist
===ignore===
TODO
