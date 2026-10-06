===description===
Negative control for the enum-case/interface subtype fix: an enum-case
literal must still be rejected when passed to a parameter typed as an
interface its declaring enum does NOT implement.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
  <phpVersion>8.1</phpVersion>
</mir>
===file===
<?php
interface Unrelated {
    public function nope(): void;
}

enum Status {
    case Active;
    case Inactive;
}

function needsUnrelated(Unrelated $x): void {}

function test(Status $s): void {
    if ($s === Status::Active) {
        needsUnrelated($s);
//                     ^^ InvalidArgument: Argument $x of needsUnrelated() expects 'Unrelated', got 'Status::Active'
    }
}
