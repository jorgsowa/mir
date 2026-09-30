===description===
Trait redefinition
===file===
<?php
trait Foo {}
trait Foo {}
//<^^^^^^^^^^^^ DuplicateTrait: Trait Foo has already been defined
===expect===
