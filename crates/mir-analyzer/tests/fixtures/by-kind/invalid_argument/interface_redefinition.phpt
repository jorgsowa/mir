===description===
Interface redefinition
===file===
<?php
interface Foo {}
interface Foo {}
===expect===
DuplicateInterface@3:0-3:16: Interface Foo has already been defined
