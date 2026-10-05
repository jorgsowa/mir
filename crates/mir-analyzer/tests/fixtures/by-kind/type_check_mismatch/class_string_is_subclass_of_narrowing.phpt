===description===
is_subclass_of() and is_a(..., true) narrow a class-string to class-string<Target>; a
class-string<Parent> narrows to the subtype, an unrelated class-string<Other> is left alone for
interfaces and a bare string is not narrowed.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
interface Contract {}
class Parent_ {}
class Impl extends Parent_ implements Contract {}
class Other {}

/** @param class-string $c */
function bare(string $c): void {
    if (is_subclass_of($c, Contract::class)) {
        /** @mir-check $c is class-string<Contract> */
        $_ = $c;
    }
    if (is_a($c, Contract::class, true)) {
        /** @mir-check $c is class-string<Contract> */
        $_ = $c;
    }
    if (is_subclass_of($c, 'Contract')) {
        /** @mir-check $c is class-string<Contract> */
        $_ = $c;
    }
}

/** @param class-string<Parent_> $c */
function parent_bound(string $c): void {
    if (is_subclass_of($c, Impl::class)) {
        /** @mir-check $c is class-string<Impl> */
        $_ = $c;
    }
    if (is_a($c, Impl::class, true)) {
        /** @mir-check $c is class-string<Impl> */
        $_ = $c;
    }
}

/** @param class-string<Impl> $c */
function already_narrower(string $c): void {
    if (is_subclass_of($c, Parent_::class)) {
        /** @mir-check $c is class-string<Impl> */
        $_ = $c;
    }
}

function plain_string(string $s): void {
    if (is_a($s, Contract::class, true)) {
        /** @mir-check $s is string */
        $_ = $s;
    }
}
===expect===
