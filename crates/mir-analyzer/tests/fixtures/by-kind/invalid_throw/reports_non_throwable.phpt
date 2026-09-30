===description===
reports non throwable
===file===
<?php
class NotAnException {}

function test(): void {
    throw new NotAnException();
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidThrow: Thrown type 'NotAnException' does not extend Throwable
}
===expect===
