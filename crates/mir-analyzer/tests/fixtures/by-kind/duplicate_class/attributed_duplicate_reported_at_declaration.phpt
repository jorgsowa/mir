===description===
An attributed duplicate is reported at its declaration, not at the attribute line.
===file===
<?php
class Foo {}
#[\AllowDynamicProperties]
class Foo {}
//<^^^^^^^^^^^^ DuplicateClass: Class Foo has already been defined
