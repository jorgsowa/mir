===description===
Invalid throw class
===file===
<?php
class A {}
throw new A();
//<^^^^^^^^^^^^^^ InvalidThrow: Thrown type 'A' does not extend Throwable
===expect===
