===description===
// line comment with @psalm-suppress above a statement suppresses it
===file===
<?php
function test(): void {
    // @psalm-suppress UndefinedClass
    new NoSuchClass();
}
===expect===
