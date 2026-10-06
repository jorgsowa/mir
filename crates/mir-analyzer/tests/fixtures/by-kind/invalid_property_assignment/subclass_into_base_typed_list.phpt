===description===
Subclasses of `Argument` belong in a `list<Argument>` (covariant element assignment).
===config===
<mir>
  <phpVersion>8.4</phpVersion>
</mir>
===file===
<?php
abstract class Argument {}
final class CommandArgument extends Argument {}
final class CommandOption extends Argument {}
class Command {
    /** @var list<Argument> */
    private array $args;
    public function __construct() {
        $this->args = [new CommandArgument(), new CommandOption()];
    }
}
