===description===
A docblock `Cls::CONST` / `Cls::*` naming another class (resolved through `use`) is a constant, not an object, so comparing it to a literal is not impossible.
===config===
suppress=UnusedParam,UnusedVariable
===file===
<?php
namespace App\Model {
    class Kind { const X = 'x'; const Y = 'y'; const N = 1; }
}

namespace App\Service {
    use App\Model\Kind;
    use App\Model\Kind as K;

    /** @param Kind::* $k */
    function wildcard(string $k): bool {
        /** @mir-check $k is App\Model\Kind::* */
        return $k === Kind::X;
    }

    /** @param Kind::X|Kind::Y $k */
    function union(string $k): bool {
        return $k === 'y';
    }

    /** @param K::N $n */
    function aliased(int $n): bool {
        return $n === 1;
    }

    /** @param \App\Model\Kind::* $k */
    function qualified(string $k): bool {
        return $k !== Kind::Y;
    }

    /** @param Kind::* $k */
    function loose(string $k): bool {
        return $k == 'x';
    }

    function genuine(string $s): bool {
        return $s === 1;
//             ^^^^^^^^ ImpossibleIdenticalComparison: '===' between 'string' and '1' is always false — these types can never be identical
    }
}
===expect===
