===description===
FP-P19: a qualified docblock class name (no leading `\`, first segment not
`use`-imported) must resolve relative to the current namespace, like PHP itself
resolves qualified names — `resolve_type_name` was early-returning it verbatim
instead. `@var Warning\Warning` inside `namespace App;` must become
`App\Warning\Warning`, not stay the literal (nonexistent) `Warning\Warning` and
get flagged UndefinedDocblockClass.
===config===
suppress=UnusedParam,MissingPropertyType
===file:Warning.php===
<?php
namespace App\Warning;

class Warning {}
===file:Container.php===
<?php
namespace App;

class Container {
    /** @var Warning\Warning */
    private $warning;
}
===expect===
