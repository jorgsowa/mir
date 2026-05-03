===description===
reports non throwable
===file===
<?php
class NotAnException {}

function test(): void {
    throw new NotAnException();
}
===expect===
InvalidThrow: Thrown type 'NotAnException' does not extend Throwable
===ignore===
TODO
