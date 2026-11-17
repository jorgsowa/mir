===description===
From PHP 8.6 `let` and `is` are deprecated as class-like, function and constant names and as use aliases, `_` as a constant name or alias, a function named `readonly`, and a class constant named `namespace`. A namespaced constant name matches only in lowercase.
===config===
<mir>
  <phpVersion>8.6</phpVersion>
</mir>
===file===
<?php
namespace App;

use Vendor\Matcher as _;
//  ^^^^^^^^^^^^^^^^^^^ DeprecatedSyntax: Using "_" as a class name is deprecated
use function Vendor\let;
//           ^^^^^^^^^^ DeprecatedSyntax: Using "let" as a function name is deprecated since 8.6
use const Vendor\FLAG as _;
//        ^^^^^^^^^^^^^^^^ DeprecatedSyntax: Using "_" as a constant name is deprecated since 8.6

class Let {}
//    ^^^ DeprecatedSyntax: Using "let" as a class name is deprecated since 8.6
interface is {}
//        ^^ DeprecatedSyntax: Using "is" as an interface name is deprecated since 8.6
function readonly(): void {}
//       ^^^^^^^^ DeprecatedSyntax: Calling a function "readonly" is deprecated since 8.6
const IS = 1;
const is = 2;
//    ^^ DeprecatedSyntax: Calling a constant "is" is deprecated since 8.6

class Pattern {
    const namespace = 'app';
//        ^^^^^^^^^ DeprecatedSyntax: Declaring class constant called 'namespace' is deprecated
    const let = 'allowed';

    public function is(): bool {
        return true;
    }
}
