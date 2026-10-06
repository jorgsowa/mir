===description===
FP-E: Inside a trait method, $this->prop declared in the same trait must
resolve to its declared type, not mixed. UndefinedProperty must not be emitted.
===config===
<mir>
  <phpVersion>8.2</phpVersion>
</mir>
===file===
<?php

trait LockableTrait {
    private bool $locked = false;

    public function isLocked(): bool {
        return $this->locked;
    }
}

class Service {
    use LockableTrait;
}
