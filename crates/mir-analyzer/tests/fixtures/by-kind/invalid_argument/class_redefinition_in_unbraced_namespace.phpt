===description===
Class redefinition in unbraced namespace
===file===
<?php
namespace A;
class Foo {}
class Foo {}
===expect===
DuplicateClass@4:1-4:13: Class A\Foo has already been defined
