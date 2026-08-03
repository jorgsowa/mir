===description===
M22: `@throws void` (the pseudo-type meaning "doesn't throw") inside a
namespaced file must not be namespace-qualified into a bogus class name
before the pseudo-type check runs — a bare-namespace file already worked;
this pins the namespaced case. Also confirms an override correctly
narrowing away the parent's real declared @throws (via `@throws void`)
doesn't inherit it back as a phantom throw.
===config===
suppress=UnusedParam
===file===
<?php
namespace App;

class Foo {
    /** @throws void */
    public function safe(): void {}
}

abstract class Base {
    /** @throws \RuntimeException */
    public function risky(): string { throw new \RuntimeException(); }
}

final class Overridden extends Base {
    /** @throws void */
    public function risky(): string { return 'ok'; }
}

function useOverride(Overridden $o): void {
    $o->risky();
}
===expect===
