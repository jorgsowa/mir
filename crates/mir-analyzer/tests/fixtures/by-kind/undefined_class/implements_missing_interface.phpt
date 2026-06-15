===description===
implements missing interface
===file===
<?php
class Bar implements MissingInterface {}
===expect===
UndefinedClass@2:21-2:37: Class MissingInterface does not exist
