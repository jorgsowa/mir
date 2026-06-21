===description===
FP-E: Inside a trait method, $this->prop declared in the same trait must
resolve to its declared type, not mixed. UndefinedProperty must not be emitted.
===config===
php_version=8.2
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
===expect===
MissingConstructor@11:0-11:15: Class Service has uninitialized properties but no constructor
