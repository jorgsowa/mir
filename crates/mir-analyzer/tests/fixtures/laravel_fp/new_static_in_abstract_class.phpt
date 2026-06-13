===description===
Regression (laravel/framework): `new static` inside an abstract class (Model,
Factory) is valid late static binding — it constructs the concrete subclass at
runtime, not the abstract class. mir no longer emits AbstractInstantiation for
`new static` (only for `new self` / `new AbstractName`).
===config===
suppress=MissingClosureReturnType,UnusedParam,UnusedVariable,UnusedFunction,MixedReturnStatement
===file===
<?php
abstract class Model {
    public static function make(): static {
        return new static();
    }
    abstract public function table(): string;
}
===expect===
