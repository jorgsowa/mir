===description===
Prevent implementing class
===file===
<?php
class Foo {}

class Bar implements Foo {}
//                   ^^^ CompileError: Bar cannot implement Foo - it is not an interface [implements-non-interface]
