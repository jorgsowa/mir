===description===
Regression (laravel/framework): the `[$this, 'method']` callable-array form is
valid `callable`. mir now accepts a 2-element `[object|string, string]` shape
against a callable parameter (array_walk), so it no longer emits InvalidArgument.
===config===
<mir>
  <issueHandlers>
    <MissingClosureReturnType errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
    <MixedArgument errorLevel="suppress"/>
    <MixedReturnStatement errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class TagSet {
    /** @var array<int, string> */
    public array $names = [];

    public function reset(): void {
        array_walk($this->names, [$this, 'resetTag']);
    }

    public function resetTag(string $name): void {}
}
===expect===
