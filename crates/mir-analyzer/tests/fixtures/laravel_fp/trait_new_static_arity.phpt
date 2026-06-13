===description===
Laravel FP (laravel/framework): `new static($items)` inside a trait
(EnumeratesValues::make) resolves `static` to the using class's constructor, not
the trait's (which has none). mir emits TooManyArguments. Ignored pending fix —
see ROADMAP §1.4 (trait-context resolution).
===ignore===
===config===
suppress=MissingPropertyType,MissingClosureReturnType,UnusedParam,UnusedVariable,UnusedFunction,MixedArgument,MixedReturnStatement
===file===
<?php
trait MakesItems {
    public static function make(array $items): static {
        return new static($items);
    }
}
class Collection {
    use MakesItems;
    public function __construct(array $items) {}
}
===expect===
