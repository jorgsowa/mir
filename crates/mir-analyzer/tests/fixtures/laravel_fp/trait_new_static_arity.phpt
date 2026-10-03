===description===
Regression (laravel/framework): `new static($items)` inside a trait
(EnumeratesValues::make) resolves `static` to the using class's constructor, not
the trait's (which has none). mir no longer validates constructor args for
`new static`/`new self`/`new parent` inside a trait, so no TooManyArguments.
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
