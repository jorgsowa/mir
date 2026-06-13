===description===
Laravel FP (laravel/framework): `static::CONST` inside a trait (e.g.
HasTimestamps::CREATED_AT) is defined on the using model via late static binding.
mir emits UndefinedConstant. Ignored pending fix — see ROADMAP §1.4.
===ignore===
===config===
suppress=MissingPropertyType,MissingClosureReturnType,UnusedParam,UnusedVariable,UnusedFunction,MixedArgument,MixedReturnStatement
===file===
<?php
trait HasTimestamps {
    public function createdAtColumn(): string {
        return static::CREATED_AT;
    }
}
class Post {
    use HasTimestamps;
    const CREATED_AT = 'created_at';
}
===expect===
