===description===
Laravel FP (laravel/framework): a foreach value used as an array key in
`unset($arr[$value])` is a use, but mir does not count an unset() dimension key as
a read and emits UnusedForeachValue (e.g. Container::forgetScopedInstances).
Ignored pending fix — see ROADMAP §1.4 (liveness read-miss).
===ignore===
===config===
suppress=MissingPropertyType,MissingClosureReturnType,UnusedParam,UnusedVariable
===file===
<?php
class Container {
    /** @var array<string, mixed> */
    protected array $instances = [];

    /** @param list<string> $scopes */
    public function forget(array $scopes): void {
        foreach ($scopes as $scoped) {
            unset($this->instances[$scoped]);
        }
    }
}
===expect===
