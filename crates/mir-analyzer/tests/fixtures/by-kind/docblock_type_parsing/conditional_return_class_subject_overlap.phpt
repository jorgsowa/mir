===description===
A class-subject conditional picks the else branch only when the argument is provably disjoint from the subject; a supertype or overlapping interface yields the union of both branches.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
interface Source {}
interface Other {}
final class DirSource implements Source {}
class Plain implements Source {}
class Unrelated {}
final class Sealed {}

final class Resolver {
    /** @return ($source is DirSource ? string|null : string) */
    public function byFinal(Source $source): ?string { return null; }

    /** @return ($source is Source ? int : string) */
    public function byInterface(object $source): int|string { return 1; }

    /** @return ($source is Plain ? int : string) */
    public function byClass(Other $source): int|string { return 1; }
}

function check(Source $s, DirSource $d, Plain $p, Unrelated $u, Sealed $x, Other $o, Resolver $r): void {
    // supertype argument may be a DirSource
    /** @mir-check $r->byFinal($s) is string|null */
    $r->byFinal($s);
    // subtype argument
    /** @mir-check $r->byFinal($d) is string|null */
    $r->byFinal($d);
    // sibling implementation of the same interface
    /** @mir-check $r->byFinal($p) is string */
    $r->byFinal($p);
    // final class not implementing the subject
    /** @mir-check $r->byInterface($x) is string */
    $r->byInterface($x);
    // non-final class not implementing the interface subject: a subclass could
    /** @mir-check $r->byInterface($u) is int|string */
    $r->byInterface($u);
    // interface vs non-final class: an implementation could extend the class
    /** @mir-check $r->byClass($o) is int|string */
    $r->byClass($o);
    // narrowed result is still usable
    $res = $r->byFinal($s);
    if ($res === null) {
        return;
    }
}
