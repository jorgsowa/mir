===description===
Interface redefinition
===file===
<?php
interface Foo {}
interface Foo {}
===expect===
DuplicateInterface@3:1-3:17: Interface Foo has already been defined
