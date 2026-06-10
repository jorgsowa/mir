===description===
Enum redefinition
===file===
<?php
enum Foo {}
enum Foo {}
===expect===
DuplicateEnum@3:1-3:12: Enum Foo has already been defined
