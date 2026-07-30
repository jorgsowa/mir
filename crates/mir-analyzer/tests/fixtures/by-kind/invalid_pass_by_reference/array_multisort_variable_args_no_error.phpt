===description===
The common form — passing plain variables to array_multisort() — must not
regress once the lenient-by-ref special case is added.
===file===
<?php
function test(array $keys, array $data): void {
    array_multisort($keys, SORT_ASC, $data);
}
===expect===
