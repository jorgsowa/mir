===description===
implements missing interface
===file===
<?php
class Bar implements MissingInterface {}
===expect===
UndefinedClass: Class MissingInterface does not exist
===ignore===
TODO
