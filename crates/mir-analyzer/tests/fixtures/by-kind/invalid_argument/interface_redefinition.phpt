===description===
Interface redefinition
===file===
<?php
interface Foo {}
interface Foo {}
//<^^^^^^^^^^^^^^^^ DuplicateInterface: Interface Foo has already been defined
===expect===
