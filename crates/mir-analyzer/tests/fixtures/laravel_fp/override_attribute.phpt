===description===
Regression (laravel/framework): the built-in PHP 8.3 attribute `#[\Override]`,
applied inside a namespace, must keep its leading-\ (global) resolution. mir was
dropping the leading backslash and re-resolving against the file namespace
(→ App\Console\Override), yielding UndefinedAttributeClass; attribute-name
resolution now honors the FullyQualified name kind.
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
