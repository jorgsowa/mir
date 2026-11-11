===description===
An optional shape property may be absent but not mistyped; inherited properties count.
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
class Node {
    public int $id = 0;
}

final class Folder extends Node {
    public string $title = '';
}

final class Draft extends Node {
    public int $title = 0;
}

/** @param object{id: int, title?: string} $item */
function index(object $item): void {}

index(new Node());
index(new Folder());
index(new Draft());
//    ^^^^^^^^^^^ InvalidArgument: Argument $item of index() expects 'object{id: int, title?: string}', got 'Draft'
