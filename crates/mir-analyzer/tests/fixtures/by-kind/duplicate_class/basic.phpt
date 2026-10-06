===description===
DuplicateClass fires when the same class is declared twice in the same file.
===file===
<?php
class Foo {}
class Foo {}
//<^^^^^^^^^^^^ DuplicateClass: Class Foo has already been defined
