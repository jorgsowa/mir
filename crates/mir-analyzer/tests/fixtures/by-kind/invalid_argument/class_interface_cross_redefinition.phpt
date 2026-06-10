===description===
Cross-kind redefinition: class and interface share one PHP symbol namespace
===file===
<?php
class Foo {}
interface Foo {}
===expect===
DuplicateInterface@3:1-3:17: Interface Foo has already been defined
