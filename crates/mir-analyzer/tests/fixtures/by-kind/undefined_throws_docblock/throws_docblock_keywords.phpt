===description===
`@throws` entries that are docblock keywords are filtered before the
class-existence check, so keyword throw specs are not reported as
undefined classes. (`void` — the only semantically common one — is
pinned separately in throws_void_in_namespaced_file_not_flagged.)
===file===
<?php
/**
 * @throws boolean
 * @throws list
 * @throws iterable
 */
function risky(): string {
    return 'x';
}

===expect===
