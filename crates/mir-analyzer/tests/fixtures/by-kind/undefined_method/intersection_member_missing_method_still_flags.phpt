===description===
Negative counterpart of the FQ-global intersection-member fix: a method
genuinely missing from every intersection member (both the global class and
a same-namespace one) must still flag UndefinedMethod — the
`resolve_receiver_fqcn` literal-first check must not mask real gaps.
===file===
<?php
namespace {
    class Glob {
        public function run(): int { return 1; }
    }
}

namespace App {
    interface Foo {}

    /**
     * @param Foo&\Glob $x
     */
    function f($x): void {
        $x->missing();
    }
}
===expect===
UndefinedMethod@15:8-15:21: Method App\Foo&Glob::missing() does not exist
