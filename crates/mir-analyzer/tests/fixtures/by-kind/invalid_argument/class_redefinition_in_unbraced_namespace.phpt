===description===
Class redefinition in unbraced namespace
===file===
<?php
namespace A;
class Foo {}
class Foo {}
//<^^^^^^^^^^^^ DuplicateClass: Class A\Foo has already been defined
===expect===
