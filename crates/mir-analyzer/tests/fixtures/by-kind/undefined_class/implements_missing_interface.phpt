===description===
implements missing interface
===file===
<?php
class Bar implements MissingInterface {}
===expect===
UndefinedClass@2:22: Class MissingInterface does not exist
