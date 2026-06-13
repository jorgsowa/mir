===description===
Laravel FP (laravel/framework): the built-in PHP 8.2 attribute
`#[\SensitiveParameter]` is missing from mir's global stubs, and inside a namespace
its leading-\ (global) name is re-resolved against the file namespace, yielding
UndefinedAttributeClass. Ignored pending fix — see ROADMAP §1.4.
===ignore===
===config===
suppress=MissingClosureReturnType,UnusedParam,UnusedVariable,MixedReturnStatement
===file===
<?php
namespace App\Encryption;

class Encrypter {
    public function decrypt(#[\SensitiveParameter] string $payload): string {
        return $payload;
    }
}
===expect===
