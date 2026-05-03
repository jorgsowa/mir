===description===
enum implements missing interface
===file===
<?php
enum Status: string implements MissingInterface {}
===expect===
UndefinedClass: Class MissingInterface does not exist
===ignore===
TODO
