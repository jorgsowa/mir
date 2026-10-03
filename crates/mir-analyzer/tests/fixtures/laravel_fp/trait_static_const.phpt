===description===
Regression (laravel/framework): `static::CONST` inside a trait (e.g.
HasTimestamps::CREATED_AT) is defined on the using model via late static binding.
mir no longer emits UndefinedConstant for self/static const access inside a trait.
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
