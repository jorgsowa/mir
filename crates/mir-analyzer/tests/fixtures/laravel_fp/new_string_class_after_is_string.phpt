===description===
Laravel FP (laravel/framework): `new $this->job()` inside `if (is_string($this->job))`
is a valid dynamic class instantiation. The MissingConstructor half was fixed by
untyped `@var` properties no longer being treated as uninitialized. The
InvalidStringClass half was blocked by a php-rs-parser 0.17 bug where `new $this->job()`
was parsed as `(new $this)->job()` instead of `new ($this->job)()`. Fixed in
php-rs-parser 0.18.0 via `parse_new_variable_tail`, which correctly handles
member-access class references.
===config===
suppress=MissingPropertyType,MissingClosureReturnType,UnusedParam,UnusedVariable,MixedReturnStatement,MixedMethodCall
===file===
<?php
class PendingChain {
    /** @var object|string */
    public $job;

    public function instance(): object {
        if (is_string($this->job)) {
            return new $this->job();
        }
        return $this->job;
    }
}
===expect===
