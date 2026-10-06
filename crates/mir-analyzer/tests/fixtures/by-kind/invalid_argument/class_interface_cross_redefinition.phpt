===description===
Cross-kind redefinition: class and interface share one PHP symbol namespace
===file===
<?php
class Foo {}
interface Foo {}
//<^^^^^^^^^^^^^^^^ DuplicateInterface: Interface Foo has already been defined
