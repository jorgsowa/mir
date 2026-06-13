===description===
Laravel FP (laravel/framework): inside a trait, `$this` is the using class (which
implements the required interface), but mir types it as the trait, so passing
`$this` where the interface is expected yields a spurious InvalidArgument.
Ignored pending fix — see ROADMAP §1.4 (trait-context resolution).
===ignore===
===config===
suppress=MissingPropertyType,MissingClosureReturnType,UnusedParam,UnusedVariable,UnusedFunction,MixedArgument,MixedReturnStatement
===file===
<?php
interface Store {}
class CacheLock {
    public function __construct(Store $store) {}
}
trait HasCacheLock {
    public function lock(): CacheLock {
        return new CacheLock($this);
    }
}
class FileStore implements Store {
    use HasCacheLock;
}
===expect===
