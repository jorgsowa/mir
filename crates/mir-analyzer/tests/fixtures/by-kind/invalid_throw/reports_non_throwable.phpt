===description===
reports non throwable
===file===
<?php
class NotAnException {}

function test(): void {
    throw new NotAnException();
}
===expect===
InvalidThrow@5:4-5:31: Thrown type 'NotAnException' does not extend Throwable
