===description===
Laravel FP (laravel/framework): `new $this->job()` inside `if (is_string($this->job))`
is a valid dynamic class instantiation. The MissingConstructor half of this FP is
fixed (an untyped `@var` property is no longer treated as uninitialized). The
InvalidStringClass half remains BLOCKED by an upstream php-rs-parser 0.17 bug:
`parse_new_expr` consumes only `$this` as the class reference for `new $this->job()`,
parsing it as `(new $this)->job()` instead of `new ($this->job)()`. mir then
correctly reports `new $this` (an object) as InvalidStringClass. Fixing this needs
a parser that handles member-access class references (`new $var->prop`); kept
ignored until then.
===ignore===
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
