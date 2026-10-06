===description===
Regression (laravel/framework): `parent::` inside a trait resolves against the
using class at runtime, not the trait. mir no longer emits ParentNotFound when
the enclosing scope is a trait.
===config===
<mir>
  <issueHandlers>
    <MissingPropertyType errorLevel="suppress"/>
    <MissingClosureReturnType errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedFunction errorLevel="suppress"/>
    <MixedArgument errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <MixedMethodCall errorLevel="suppress"/>
    <MixedReturnStatement errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Base {
    public function boot(): void {}
}
trait HasBoot {
    public function init(): void {
        parent::boot();
    }
}
class Widget extends Base {
    use HasBoot;
}
