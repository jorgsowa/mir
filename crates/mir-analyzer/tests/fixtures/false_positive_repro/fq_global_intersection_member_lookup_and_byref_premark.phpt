===description===
FP-L20: a fully-qualified GLOBAL class name (`\Glob`, no namespace) used as
an intersection-type member, referenced from inside a namespaced file, broke
member lookup. `db::resolve_receiver_fqcn` (called on the already-canonical
`fqcn` an intersection part's `TNamedObject` atom carries) re-ran
`resolve_name`'s raw-source-text rules on it, which can't tell "already
resolved" apart from "bare unqualified text needing the current namespace" —
so the already-correct bare "Glob" got wrongly rewritten to "App\Glob",
which doesn't exist. Same root cause broke by-ref out-param premarking on an
intersection receiver (`named_object_fqcn()` returning `None` for
`TIntersection` skipped it outright).
===config===
suppress=UnusedParam,UnusedVariable
===file===
<?php
namespace {
    class Glob {
        public function run(): int { return 1; }
        public function fill(&$out): void { $out = 1; }
    }
}

namespace App {
    interface Foo {}

    /**
     * @param Foo&\Glob $x
     */
    function callsMethod($x): int {
        return $x->run();
    }

    /**
     * @param Foo&\Glob $x
     */
    function premarksByRefOutParam($x): void {
        $x->fill($out);
        echo $out;
    }
}
===expect===
