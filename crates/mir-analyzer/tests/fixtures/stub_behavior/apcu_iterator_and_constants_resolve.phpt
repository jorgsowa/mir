===description===
The hand-written apcu stub lacked `APCUIterator`, the `APC_ITER_*`/`APC_LIST_*`
constants and `apcu_cas`/`apcu_entry`/`apcu_enabled`/`apcu_key_info`, all of
which PhpStormStubsMap.php routes to `apcu/apcu.php`.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php

function warm(): int {
    $value = apcu_entry('k', fn(string $key): int => strlen($key));
    $swapped = apcu_cas('k', 1, 2);
    /** @mir-check $value is int */
    /** @mir-check $swapped is bool */
    $info = apcu_key_info('k');
    /** @mir-check $info is array|null */
    $it = new APCUIterator('/^user\./', APC_ITER_KEY | APC_ITER_VALUE, 100, APC_LIST_ACTIVE);
    $total = $it->getTotalCount();
    /** @mir-check $total is int */
    return apcu_enabled() ? $total : 0;
}
