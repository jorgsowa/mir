===description===
A class instance fits `object{...}` only when it declares each required property with a compatible type.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedProperty errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
final class Celsius {
    public float $degrees = 0.0;
}

final class Label {
    public string $degrees = 'warm';
}

final class Empty_ {
    public int $other = 1;
}

/** @param object{degrees: float} $reading */
function record(object $reading): void {}

record(new Celsius());
record(new Label());
//     ^^^^^^^^^^^ InvalidArgument: Argument $reading of record() expects 'object{degrees: float}', got 'Label'
record(new Empty_());
//     ^^^^^^^^^^^^ InvalidArgument: Argument $reading of record() expects 'object{degrees: float}', got 'Empty_'
