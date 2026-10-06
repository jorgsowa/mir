===description===
PHP_INT_SIZE retains its 4|8 literal union for typed returns and comparisons:
both real platform widths remain possible, while an unsupported width is
correctly impossible.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @return 4|8 */
function wordSize(): int {
    return PHP_INT_SIZE;
}

function supportsCurrentPlatform(): bool {
    return PHP_INT_SIZE === 4 || PHP_INT_SIZE === 8;
}

function impossiblePlatform(): bool {
    return PHP_INT_SIZE === 16;
//         ^^^^^^^^^^^^^^^^^^^ ImpossibleIdenticalComparison: '===' between '4|8' and '16' is always false — these types can never be identical
}
