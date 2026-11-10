===description===
A ternary condition already explained by an impossible comparison gets no second RedundantCondition.
===file===
<?php
function compare(int $code): string {
    return $code === 'ok' ? 'success' : 'failure';
//         ^^^^^^^^^^^^^^ ImpossibleIdenticalComparison: '===' between 'int' and '"ok"' is always false — these types can never be identical
}
