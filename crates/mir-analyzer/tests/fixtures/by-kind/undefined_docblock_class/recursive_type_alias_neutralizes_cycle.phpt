===description===
A recursive @psalm-type alias (self-reference through a container) only
unrolled one level, then left a dangling reference to a phantom,
nonexistent class named after the alias itself — `expand_type_aliases_
fixpoint` runs exactly `aliases.len()` passes (1, for a single
self-referential alias), which is enough to resolve a finite CHAIN of
aliases but can never fully expand a genuine cycle. A final pass now
neutralizes any alias-name atom still present after that many passes
(which can only be cyclic residue) to `mixed` instead of leaving it
dangling.
===config===
suppress=UnusedParam
===file===
<?php
/**
 * @psalm-type Tree = array{value: int, children: array<Tree>}
 */
class TreeHolder {
    /** @param Tree $t */
    public function process($t): void {
        /** @mir-check $t is array{value: int, children: array<array{value: int, children: array<mixed>}>} */
        echo "ok";
    }
}
===expect===
