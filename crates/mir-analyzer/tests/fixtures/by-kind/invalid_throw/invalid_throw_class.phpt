===description===
Invalid throw class
===file===
<?php
class A {}
throw new A();
===expect===
InvalidThrow@3:1-3:15: Thrown type 'A' does not extend Throwable
