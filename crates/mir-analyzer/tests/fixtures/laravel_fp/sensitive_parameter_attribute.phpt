===description===
Regression (laravel/framework): the built-in PHP 8.2 attribute
`#[\SensitiveParameter]`, applied inside a namespace, must keep its leading-\
(global) resolution. mir was dropping the leading backslash and re-resolving
against the file namespace (→ App\Encryption\SensitiveParameter), yielding
UndefinedAttributeClass; attribute-name resolution now honors the FullyQualified
name kind.
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
