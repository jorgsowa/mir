===description===
FP-P19 (param form): a namespace-relative qualified docblock class name in a
method's `@param` tag must resolve against the current namespace instead of
staying the literal (nonexistent) `Warning\Warning` and getting flagged
UndefinedDocblockClass. Uses an untyped param so the method-level check reuses
the collector-resolved stored type directly (a native-hinted param takes a
different, already-correct db-level re-resolution path and wouldn't exercise
this bug).
===config===
suppress=UnusedParam
===file:Warning.php===
<?php
namespace App\Warning;

class Warning {}
===file:Container.php===
<?php
namespace App;

class Container {
    /** @param Warning\Warning $warning */
    public function set($warning): void {}
}
===expect===
