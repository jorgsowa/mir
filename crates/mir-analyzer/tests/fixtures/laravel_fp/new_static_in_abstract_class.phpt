===description===
Laravel FP (laravel/framework): `new static` inside an abstract class (Model,
Factory) is valid late static binding — it constructs the concrete subclass at
runtime, not the abstract class. mir emits AbstractInstantiation. Ignored pending
fix — see ROADMAP §1.4.
===ignore===
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
