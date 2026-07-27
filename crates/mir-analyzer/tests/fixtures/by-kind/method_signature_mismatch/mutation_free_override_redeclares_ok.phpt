===description===
Contrast case: an override that DOES re-declare @mutation-free on itself
must stay unflagged.
===file===
<?php
interface Counter {
    /** @mutation-free */
    public function peek(): int;
}
class StillMutationFree implements Counter {
    /** @mutation-free */
    public function peek(): int {
        return 1;
    }
}
===expect===
