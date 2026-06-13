===description===
Laravel FP (laravel/framework): the `[$this, 'method']` callable-array form is
valid `callable`, but mir types it as array{0: X, 1: "method"} and rejects it
against a callable parameter (array_walk), emitting InvalidArgument. Ignored
pending fix — see ROADMAP §1.4.
===ignore===
===config===
suppress=MissingClosureReturnType,UnusedParam,UnusedVariable,MixedArgument,MixedReturnStatement
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
