===description===
Contrast case: an override that DOES re-declare @external-mutation-free
on itself must stay unflagged.
===file===
<?php
interface Cache {
    /** @external-mutation-free */
    public function bump(): int;
}
class StillExternalMutationFree implements Cache {
    /** @external-mutation-free */
    public function bump(): int {
        return 1;
    }
}
===expect===
