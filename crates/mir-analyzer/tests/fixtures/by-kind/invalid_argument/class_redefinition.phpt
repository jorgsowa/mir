===description===
Class redefinition
===file===
<?php
class Foo {}
class Foo {}
===expect===
DuplicateClass@3:1-3:13: Class Foo has already been defined
