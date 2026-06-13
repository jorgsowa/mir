===description===
Regression (laravel/framework): a foreach value used as an array key in
`unset($arr[$value])` is a use. mir now analyzes non-variable unset targets, so
the dimension key counts as a read and UnusedForeachValue is no longer emitted
(e.g. Container::forgetScopedInstances).
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
