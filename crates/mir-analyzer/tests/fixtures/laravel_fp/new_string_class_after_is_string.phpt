===description===
Laravel FP (laravel/framework): `new $this->job()` inside `if (is_string($this->job))`
is a valid dynamic class instantiation, but mir does not narrow on is_string()
(and resolves $this->job to its property type), emitting InvalidStringClass.
Ignored pending fix — see ROADMAP §1.4.
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
