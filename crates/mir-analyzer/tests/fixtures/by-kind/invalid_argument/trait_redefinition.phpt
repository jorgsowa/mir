===description===
Trait redefinition
===file===
<?php
trait Foo {}
trait Foo {}
===expect===
DuplicateTrait@3:0-3:12: Trait Foo has already been defined
