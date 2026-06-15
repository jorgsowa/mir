===description===
Class redefinition
===file===
<?php
class Foo {}
class Foo {}
===expect===
DuplicateClass@3:0-3:12: Class Foo has already been defined
