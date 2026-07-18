===description===
A bare `@var` annotation referencing a class-scoped `@psalm-type` alias whose
target class doesn't exist must flag the target's name, not the alias name
itself (proves alias expansion runs before the UndefinedDocblockClass check).
===config===
suppress=MixedAssignment
===file===
<?php
/**
 * @psalm-type Result = TotallyMissingClass
 */
class Repo {
    public function find(): void {
        /** @var Result $x */
        $x = fetchSomething();
        $x->doStuff();
    }
}

function fetchSomething(): mixed {
    return null;
}
===expect===
UndefinedDocblockClass@8:8-8:30: Docblock type 'TotallyMissingClass' does not exist
