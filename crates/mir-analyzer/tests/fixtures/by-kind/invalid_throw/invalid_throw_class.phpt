===description===
Invalid throw class
===file===
<?php
class A {}
throw new A();
===expect===
InvalidThrow@3:0-3:14: Thrown type 'A' does not extend Throwable
