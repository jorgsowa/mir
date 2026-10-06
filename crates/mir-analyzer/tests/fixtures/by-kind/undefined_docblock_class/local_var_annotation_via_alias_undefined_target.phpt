===description===
A bare `@var` annotation referencing a class-scoped `@psalm-type` alias whose
target class doesn't exist must flag the target's name, not the alias name
itself (proves alias expansion runs before the UndefinedDocblockClass check).
===config===
<mir>
  <issueHandlers>
    <MixedAssignment errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/**
 * @psalm-type Result = TotallyMissingClass
 */
class Repo {
    public function find(): void {
        /** @var Result $x */
        $x = fetchSomething();
//      ^^^^^^^^^^^^^^^^^^^^^^ UndefinedDocblockClass: Docblock type 'TotallyMissingClass' does not exist
        $x->doStuff();
    }
}

function fetchSomething(): mixed {
    return null;
}
