===description===
Trait redefinition
===file===
<?php
trait Foo {}
trait Foo {}
===expect===
DuplicateTrait@3:1-3:13: Trait Foo has already been defined
