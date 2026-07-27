===description===
A `global`-declared variable's WRITE was never checked under
@mutation-free/@external-mutation-free — only @pure caught it, and only
via the one-time declaration-site check (which deliberately doesn't fire
for these two tags, since they allow reading external state and only
forbid writing it). Both the whole-variable overwrite and a property
write through a global-held object went completely unflagged.
===config===
suppress=MissingConstructor,MixedAssignment,MixedPropertyAssignment
===file===
<?php
class Bag {
    public int $count = 0;
}

class Registry {
    /** @psalm-mutation-free */
    public function corrupt(): void {
        global $counter;
        $counter = $counter + 1;
    }

    /** @psalm-external-mutation-free */
    public function corrupt2(): void {
        global $registry;
        $registry->count = 5;
    }
}
===expect===
ImpureGlobalVariable@10:8-10:31: Using global variable $counter in a @pure function
ImpurePropertyAssignment@16:8-16:28: Assigning to property count of a parameter in a pure or external-mutation-free context
