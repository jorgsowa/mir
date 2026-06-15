===description===
Cross-kind redefinition: class and interface share one PHP symbol namespace
===file===
<?php
class Foo {}
interface Foo {}
===expect===
DuplicateInterface@3:0-3:16: Interface Foo has already been defined
