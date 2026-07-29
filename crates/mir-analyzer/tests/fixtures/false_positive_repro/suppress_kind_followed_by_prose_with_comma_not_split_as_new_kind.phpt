===description===
A `@psalm-suppress` directive's trailing free-text explanation is comma-
separated prose (`... because of a legacy contract, not fully typed`), not a
second kind name. The comma inside that prose must not be mistaken for the
start of another suppressed kind — which previously produced a phantom
`UnusedSuppress` for the bogus kind "not".
===file===
<?php
function test(): void {
    /**
     * @psalm-suppress UndefinedClass because of a legacy contract, not fully typed
     */
    new NoSuchClass();
}
===expect===
