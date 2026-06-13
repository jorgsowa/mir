===description===
Laravel FP (laravel/framework): the built-in PHP 8.3 attribute `#[\Override]` is
missing from mir's global stubs, and inside a namespace its leading-\ name is
re-resolved against the file namespace, yielding UndefinedAttributeClass. Ignored
pending fix — see ROADMAP §1.4.
===ignore===
===config===
suppress=MissingClosureReturnType,UnusedParam,UnusedVariable,MixedReturnStatement
===file===
<?php
namespace App\Console;

class Base {
    public function handle(): void {}
}
class Cmd extends Base {
    #[\Override]
    public function handle(): void {}
}
===expect===
