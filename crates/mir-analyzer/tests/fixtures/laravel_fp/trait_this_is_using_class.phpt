===description===
Regression (laravel/framework): inside a trait, `$this` is the using class (which
implements the required interface). A trait-typed value is now treated as a
subtype of any object target (its concrete using class is unknown), so passing
`$this` where the interface is expected no longer yields a spurious
InvalidArgument.
===config===
<mir>
  <issueHandlers>
    <MissingPropertyType errorLevel="suppress"/>
    <MissingClosureReturnType errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedFunction errorLevel="suppress"/>
    <MixedArgument errorLevel="suppress"/>
    <MixedReturnStatement errorLevel="suppress"/>
  </issueHandlers>
</mir>
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
