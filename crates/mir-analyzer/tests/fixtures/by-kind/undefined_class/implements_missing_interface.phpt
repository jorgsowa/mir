===description===
implements missing interface
===file===
<?php
class Bar implements MissingInterface {}
//                   ^^^^^^^^^^^^^^^^ UndefinedClass: Class MissingInterface does not exist
===expect===
