===description===
`stdClass{name: string}` is read as the object shape `object{name: string}`, so its properties are typed.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
declare(strict_types=1);

function takesString(string $value): void {}
function takesInt(int $value): void {}

/** @param stdClass{name: string, age?: int} $obj */
function readsShape(\stdClass $obj): void {
    takesString($obj->name);
    takesInt($obj->name);
    //       ^^^^^^^^^^ InvalidArgument: Argument $value of takesInt() expects 'int', got 'string'
}

/** @param \stdClass{id: int} $obj */
function readsQualified(\stdClass $obj): void {
    /** @mir-check $obj->id is int */
    echo 1;
}

$valid = new \stdClass();
$valid->name = 'Ada';
readsShape($valid);
