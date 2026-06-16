===description===
FALSE POSITIVE reproducer. Valid PHP: Subclasses of `Argument` belong in a `list<Argument>` (covariant element assignment).
mir 0.42.0 currently emits (the bug): InvalidPropertyAssignment@10:8-10:66: expected list<Argument>, actual array{0:CommandArgument,1:CommandOption}
Expected: no issue. Remove ===ignore=== to activate once fixed.
===ignore===
===config===
php_version=8.4
===file===
<?php
abstract class Argument {}
final class CommandArgument extends Argument {}
final class CommandOption extends Argument {}
class Command {
    /** @var list<Argument> */
    private array $args;
    public function __construct() {
        // FP expected: InvalidPropertyAssignment (subclasses into list<Argument>)
        $this->args = [new CommandArgument(), new CommandOption()];
    }
}
===expect===
