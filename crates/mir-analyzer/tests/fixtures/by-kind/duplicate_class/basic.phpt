===description===
DuplicateClass fires when the same class is declared twice in the same file.
===file===
<?php
class Foo {}
class Foo {}
===expect===
DuplicateClass@3:1-3:13: Class Foo has already been defined
