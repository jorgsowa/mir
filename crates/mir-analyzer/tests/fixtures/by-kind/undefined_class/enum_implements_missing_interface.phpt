===description===
enum implements missing interface
===file===
<?php
enum Status: string implements MissingInterface {}
//                             ^^^^^^^^^^^^^^^^ UndefinedClass: Class MissingInterface does not exist
