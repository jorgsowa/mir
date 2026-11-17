===description===
From PHP 8.6 define() of a constant named `let`, `is` or `_` and class_alias() to `let` or `is` are deprecated; a namespaced constant name matches only in lowercase.
===config===
<mir>
  <phpVersion>8.6</phpVersion>
</mir>
===file===
<?php
define('LET', 1);
//<^^^^^^^^^^^^^^^^ DeprecatedSyntax: Calling a constant "let" is deprecated since 8.6
define('App\is', 2);
//<^^^^^^^^^^^^^^^^^^^ DeprecatedSyntax: Calling a constant "is" is deprecated since 8.6
define('App\Is', 3);
define('_', 4);
//<^^^^^^^^^^^^^^ DeprecatedSyntax: Calling a constant "_" is deprecated since 8.6
class_alias(ArrayObject::class, 'App\Is');
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ DeprecatedSyntax: Using "is" as a class alias is deprecated since 8.6
class_alias(ArrayObject::class, 'App\Matcher');
