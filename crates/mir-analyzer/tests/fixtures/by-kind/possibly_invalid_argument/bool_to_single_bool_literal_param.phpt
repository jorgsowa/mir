===description===
`bool` passed to a `true`-only or `false`-only parameter is a possibly invalid argument.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function takesFalse(false $v): void {}
function takesTrue(true $v): void {}
function takesFalseOrInt(false|int $v): void {}
function takesBool(bool $v): void {}

function f(bool $a, ?bool $b): void
{
    takesFalse($a);
    //         ^^ PossiblyInvalidArgument: Argument $v of takesFalse() expects 'false', possibly different type 'bool' provided
    takesTrue($a);
    //        ^^ PossiblyInvalidArgument: Argument $v of takesTrue() expects 'true', possibly different type 'bool' provided
    takesFalseOrInt($a);
    //              ^^ PossiblyInvalidArgument: Argument $v of takesFalseOrInt() expects 'false|int', possibly different type 'bool' provided
    takesFalse($b);
    //         ^^ PossiblyNullArgument: Argument $v of takesFalse() might be null
    //         ^^ PossiblyInvalidArgument: Argument $v of takesFalse() expects 'false', possibly different type 'bool|null' provided
    takesBool($a);
}

function narrowed(bool $a): void
{
    if (!$a) {
        takesFalse($a);
    } else {
        takesTrue($a);
    }
}
