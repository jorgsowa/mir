===description===
enum implements missing interface
===file===
<?php
enum Status: string implements MissingInterface {}
===expect===
UndefinedClass@2:31: Class MissingInterface does not exist
===ignore===
TODO
