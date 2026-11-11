===description===
`int-mask-of<Other::*>` in a free function is every combination of that class's int constants.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
namespace App\Acl;

final class Permissions {
    public const READ = 1;
    public const WRITE = 2;
    public const EXECUTE = 4;
    public const LABEL = 'permissions';
}

/** @param int-mask-of<Permissions::*> $flags */
function grant(int $flags): void {}

grant(Permissions::READ);
grant(Permissions::READ | Permissions::WRITE);
grant(0);
grant(7);
grant(8);
//    ^ InvalidArgument: Argument $flags of grant() expects '0|1|2|3|4|5|6|7', got '8'
grant(-1);
//    ^^ InvalidArgument: Argument $flags of grant() expects '0|1|2|3|4|5|6|7', got '-1'
grant('rw');
//    ^^^^ InvalidArgument: Argument $flags of grant() expects '0|1|2|3|4|5|6|7', got '"rw"'
