===description===
enum implements missing interface
===file===
<?php
enum Status: string implements MissingInterface {}
===expect===
UndefinedClass@2:31-2:47: Class MissingInterface does not exist
