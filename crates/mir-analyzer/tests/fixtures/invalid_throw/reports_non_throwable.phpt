===description===
reports non throwable
===file===
<?php
class NotAnException {}

function test(): void {
    throw new NotAnException();
}
===expect===
InvalidThrow@5:5: Thrown type 'NotAnException' does not extend Throwable
