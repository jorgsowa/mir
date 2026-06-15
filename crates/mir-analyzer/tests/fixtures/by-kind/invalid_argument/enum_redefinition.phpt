===description===
Enum redefinition
===file===
<?php
enum Foo {}
enum Foo {}
===expect===
DuplicateEnum@3:0-3:11: Enum Foo has already been defined
