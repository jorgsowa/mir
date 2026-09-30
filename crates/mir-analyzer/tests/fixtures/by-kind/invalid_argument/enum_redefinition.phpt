===description===
Enum redefinition
===file===
<?php
enum Foo {}
enum Foo {}
//<^^^^^^^^^^^ DuplicateEnum: Enum Foo has already been defined
===expect===
