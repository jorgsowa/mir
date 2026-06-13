===description===
Laravel FP (laravel/framework): `parent::` inside a trait resolves against the
using class at runtime, not the trait. mir emits ParentNotFound. Ignored pending
fix — see ROADMAP §1.4 (trait-context resolution).
===ignore===
===config===
suppress=MissingPropertyType,MissingClosureReturnType,UnusedParam,UnusedVariable,UnusedFunction,MixedArgument,MixedAssignment,MixedMethodCall,MixedReturnStatement
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
===expect===
