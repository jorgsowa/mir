===description===
single-line /** @psalm-suppress */ docblock above a statement
===file===
<?php
function test(): void {
    /** @psalm-suppress UndefinedClass */
    new NoSuchClass();
}
===expect===
